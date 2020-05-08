use crate::pg::{Table, InputTable, Network, Country};
use indicatif::ProgressBar;
use crate::stream::{GeoStream, NetStream};
use rayon::prelude::*;
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let _cpus = num_cpus::get();

    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();
    println!("ok - processing {}", &iso);

    let master_src = args.value_of("NETWORK").unwrap().to_string();

    let master = Network::new("master");
    let country = Country::new();
    println!("ok - formatted database");

    let mut manager = Vec::with_capacity(2);
    {
        let mut db = pool.get().unwrap();
        manager.push(thread::spawn(move || {
            master.create(&mut db);
            master.input(&mut db, NetStream::new(
                    GeoStream::new(Some(master_src)),
                    Some(String::from("/tmp/master_error.log")))
                        );
            master.index(&mut db);
            println!("ok - imported {} master lines", master.count(&mut db));
        }));
    }

    {
        let mut db = pool.get().unwrap();
        manager.push(thread::spawn(move || {
            if country.count(&mut db) == 0 {
                country.create(&mut db);
                country.index(&mut db);
                println!("ok - imported {} country polygons", country.count(&mut db));
            } else {
                println!("ok - country table exists");
            }
        }));
    }

    for thread in manager {
        thread.join().unwrap();
    }

    let mut db = pool.get().unwrap();
    db.execute(format!("
        DROP TABLE IF EXISTS {}_raster
    ", &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        DROP TABLE IF EXISTS {}_buffer
    ", &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        DROP TABLE IF EXISTS {}_geom
    ", &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE TABLE {}_raster AS
            SELECT
                rid,
                ST_Clip(rast, geom, true) AS rast
            FROM
                pop.population,
                country
            WHERE
                LOWER(country.iso) = LOWER($1)
                AND ST_Intersects(rast, geom)
    ", &iso).as_str(), &[&iso]).unwrap();
    println!("ok - created raster subset");

    db.execute(format!("
        ALTER TABLE {}_raster
            ADD PRIMARY KEY (rid)
    ", &iso).as_str(), &[]).unwrap();

    db.execute(r#"
        ALTER TABLE master
            ADD COLUMN geom_buff GEOMETRY(MultiPolygon, 4326)
    "#, &[]).unwrap();

    db.execute(r#"
        UPDATE master
            SET geom_buff = ST_Multi(ST_Buffer(geom::GEOGRAPHY, 2000)::GEOMETRY)
    "#, &[]).unwrap();

    db.execute(r#"
        DELETE
            FROM
                master
            WHERE
                NOT ST_IsValid(geom_buff);
    "#, &[]).unwrap();

    db.execute(r#"
        CREATE INDEX master_buff_idx
            ON master USING GIST (geom_buff)
    "#, &[]).unwrap();

    db.execute(format!("
        CREATE TABLE {iso}_geom AS
            SELECT
                gv.rid AS rid,
                (gv.geom).val AS pop,
                (gv.geom).geom AS geom
            FROM (
                SELECT
                    rid,
                    ST_PixelAsPolygons(rast) AS geom
                FROM
                    {iso}_raster
                ) gv
            WHERE
                (gv.geom).val > 0
    ", iso = &iso).as_str(), &[]).unwrap();
    println!("ok - created population areas");

    db.execute(format!("
        ALTER TABLE {iso}_geom
            ADD COLUMN coverage INT
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        ALTER TABLE {iso}_geom
            ADD COLUMN coverage_geom GEOMETRY(MULTIPOLYGON, 4326)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE INDEX {iso}_geom_gidx
            ON {iso}_geom USING GIST (geom)
    ", iso = &iso).as_str(), &[]).unwrap();


    db.execute(format!("
        ALTER TABLE {iso}_geom
            ADD COLUMN id BIGSERIAL
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE INDEX {iso}_geom_idx
            ON {iso}_geom(id)
    ", iso = &iso).as_str(), &[]).unwrap();

    let max: i64 = match db.query(format!("
        SELECT
            MAX(id)
        FROM
            {iso}_geom
    ", iso = &iso).as_str(), &[]) {
        Err(err) => panic!("{}", err),
        Ok(res) => res.get(0).unwrap().get(0)
    };

    println!("ok - calculating coverage geometry\n");

    let pb = ProgressBar::new(max as u64);

    (1..=max).into_par_iter().for_each(|i| {
        pool.get().unwrap().execute(format!("
            UPDATE {iso}_geom
                SET coverage_geom = (
                    SELECT
                        ST_Multi(ST_Union(fx.geom)) AS coverage_geom
                    FROM (
                        SELECT
                            ST_Intersection(master.geom_buff, px.geom) AS geom
                        FROM
                            master,
                            {iso}_geom px
                        WHERE
                            ST_Intersects(master.geom_buff, px.geom)
                            AND px.id = $1
                    ) fx
                )
                WHERE
                    py_geom.id = $1
        ", iso = &iso).as_str(), &[&i]).unwrap();
        pb.inc(1);
    });

    pb.finish();

    println!("\nok - done calculating coverage geometry");
}

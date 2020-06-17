use crate::pg::{Table, InputTable, Network, Country, Polygon};
use indicatif::ProgressBar;
use crate::stream::{GeoStream, NetStream, PolyStream};
use rayon::prelude::*;
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {

    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();
    println!("ok - processing {}", &iso);

    let master_src = args.value_of("NETWORK").unwrap().to_string();

    let mut db = pool.get().unwrap();

    db.execute(format!("
        CREATE SCHEMA IF NOT EXISTS country_{iso}
    ", iso = &iso).as_str(), &[]).unwrap();


    let master = Network::new(format!("country_{}.master", &iso));
    let country = Country::new(format!("country_{}.country", &iso));
    println!("ok - formatted database");

    let poly = Polygon::new(format!("country_{}.bounds", &iso));
    let mut db = pool.get().unwrap();
    poly.create(&mut db);

    match args.value_of("bounds") {
        Some(bounds) => {
            println!("ok - importing bounds file");

            poly.input(&mut db, PolyStream::new(
                GeoStream::new(Some(bounds.to_string())),
                Some(String::from("/tmp/master_error.log")))
            );
            poly.index(&mut db);

            println!("ok - imported {} bounds", poly.count());
        },
        None => ()
    };

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

    db.execute(format!("
        DROP TABLE IF EXISTS country_{iso}.{iso}_raster
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        DROP TABLE IF EXISTS country_{iso}.{iso}_buffer
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        DROP TABLE IF EXISTS country_{iso}.{iso}_geom
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE TABLE country_{iso}.{iso}_raster AS
            SELECT
                rid,
                ST_Clip(rast, geom, true) AS rast
            FROM
                pop.population,
                country_{iso}.country
            WHERE
                LOWER(country.iso) = LOWER($1)
                AND ST_Intersects(rast, geom)
    ", iso = &iso).as_str(), &[&iso]).unwrap();
    println!("ok - created raster subset");

    db.execute(format!("
        ALTER TABLE country_{iso}.{iso}_raster
            ADD PRIMARY KEY (rid)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        ALTER TABLE country_{iso}.master
            ADD COLUMN geom_buff GEOMETRY(MultiPolygon, 4326)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        UPDATE country_{iso}.master
            SET geom_buff = ST_Multi(ST_Buffer(geom::GEOGRAPHY, 2000)::GEOMETRY)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        DELETE
            FROM
                country_{iso}.master
            WHERE
                NOT ST_IsValid(geom_buff);
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE INDEX master_buff_idx
            ON country_{iso}.master USING GIST (geom_buff)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE TABLE country_{iso}.{iso}_geom AS
            SELECT
                gv.rid AS rid,
                (gv.geom).val AS pop,
                (gv.geom).geom AS geom
            FROM (
                SELECT
                    rid,
                    ST_PixelAsPolygons(rast) AS geom
                FROM
                    country_{iso}.{iso}_raster
                ) gv
            WHERE
                (gv.geom).val > 0
    ", iso = &iso).as_str(), &[]).unwrap();
    println!("ok - created population areas");

    db.execute(format!("
        ALTER TABLE country_{iso}.{iso}_geom
            ADD COLUMN coverage INT
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        ALTER TABLE country_{iso}.{iso}_geom
            ADD COLUMN coverage_geom GEOMETRY(MULTIPOLYGON, 4326)
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE INDEX {iso}_geom_gidx
            ON country_{iso}.{iso}_geom USING GIST (geom)
    ", iso = &iso).as_str(), &[]).unwrap();


    db.execute(format!("
        ALTER TABLE country_{iso}.{iso}_geom
            ADD COLUMN id BIGSERIAL
    ", iso = &iso).as_str(), &[]).unwrap();

    db.execute(format!("
        CREATE INDEX {iso}_geom_idx
            ON country_{iso}.{iso}_geom(id)
    ", iso = &iso).as_str(), &[]).unwrap();

    let max: i64 = match db.query(format!("
        SELECT
            MAX(id)
        FROM
            country_{iso}.{iso}_geom
    ", iso = &iso).as_str(), &[]) {
        Err(err) => panic!("{}", err),
        Ok(res) => res.get(0).unwrap().get(0)
    };

    println!("ok - calculating coverage geometry\n");

    let pb = ProgressBar::new(max as u64);

    (1..=max).into_par_iter().for_each(|i| {
        pool.get().unwrap().execute(format!("
            UPDATE country_{iso}.{iso}_geom
                SET coverage_geom = (
                    SELECT
                        ST_Multi(ST_Union(fx.geom)) AS coverage_geom
                    FROM (
                        SELECT
                            ST_Intersection(master.geom_buff, px.geom) AS geom
                        FROM
                            country_{iso}.master,
                            country_{iso}.{iso}_geom px
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

    db.execute(format!("
        UPDATE country_{iso}.{iso}_geom
            SET coverage = ROUND(LEAST(COALESCE(ST_Area(coverage_geom), 0.0) / ST_Area(geom), 1) * 100);
    ", iso = &iso).as_str(), &[]).unwrap();

    println!("\nok - done calculating coverage geometry");

    let covered: f64 = match db.query(format!("
        SELECT
            SUM(pop * coverage * 0.01)
        FROM
            country_{iso}.{iso}_geom
    ", iso = &iso).as_str(), &[]) {
        Err(err) => panic!("{}", err),
        Ok(res) => res.get(0).unwrap().get(0)
    };

    let uncovered: f64 = match db.query(format!("
        SELECT
            SUM(pop) - SUM(pop * coverage * 0.01)
        FROM
            country_{iso}.{iso}_geom
    ", iso = &iso).as_str(), &[]) {
        Err(err) => panic!("{}", err),
        Ok(res) => res.get(0).unwrap().get(0)
    };

    println!("Country:");
    println!("Covered: {}", covered);
    println!("Uncovered: {}", uncovered);

    if poly.count() > 0 {
        let covered: f64 = match db.query(format!("
            SELECT
                country_{iso}.name,
                SUM(pop * coverage * 0.01)
            FROM
                country_{iso}.{iso}_geom,
                country_{iso}.bounds
            WHERE
                ST_Intersects(country_{iso}.geom, ountry_{iso}.{iso}_geom.geom)
        ", iso = &iso).as_str(), &[]) {
            Err(err) => panic!("{}", err),
            Ok(res) => res.get(0).unwrap().get(0)
        };
    }
}

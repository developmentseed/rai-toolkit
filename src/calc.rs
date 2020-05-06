use crate::pg::{Table, InputTable, Network, Country};
use crate::stream::{GeoStream, NetStream};
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
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

    //db.query(format!("
    //    UPDATE {}_raster
    //        SET rast = ST_AddBand(rast, '1BB'::text, 0)
    //", &iso).as_str(), &[]).unwrap();
}

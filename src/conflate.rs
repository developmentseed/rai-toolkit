use crate::pg::{Table, InputTable, Network};
use crate::{Tokens, Tokenized};
use crate::stream::{GeoStream, NetStream};
use rayon::prelude::*;
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();
    let langs: Vec<String> = args.value_of("langs").unwrap().to_string().to_lowercase().split(',').map(|i| {
        String::from(i.trim())
    }).collect();

    let abbr = Tokens::generate(langs);

    let master_src = args.value_of("MASTER").unwrap().to_string();
    let new_src = args.value_of("NEW").unwrap().to_string();

    let master = Network::new("master");
    let new = Network::new("new");

    println!("ok - formatted database");

    let mut manager = Vec::with_capacity(2);
    {
        let mut db = pool.get().unwrap();
        let master = master.clone();
        manager.push(thread::spawn(move || {
            master.create(&mut db);
            master.input(&mut db, NetStream::new(
                GeoStream::new(Some(master_src)),
                Some(String::from("/tmp/master_error.log")))
            );
            master.index(&mut db);
            master.seq(&mut db);
            println!("ok - imported {} master lines", master.count(&mut db));

        }));
    }

    {
        let mut db = pool.get().unwrap();
        let new = new.clone();
        manager.push(thread::spawn(move || {
            new.create(&mut db);
            new.input(&mut db, NetStream::new(
                GeoStream::new(Some(new_src)),
                Some(String::from("/tmp/new_error.log")))
            );
            new.index(&mut db);
            new.seq(&mut db);
            println!("ok - imported {} new lines", new.count(&mut db));

        }));
    }

    for thread in manager {
        thread.join().unwrap();
    }

    let master_max = master.max(&mut pool.get().unwrap()).unwrap();

    (1..=master_max).into_par_iter().for_each(|i| {
        let mut db = pool.get().unwrap();

        let props = master.props(&mut db, i);

        if !props.contains_key("name") {
            return;
        }

        let names: Vec<Vec<Tokenized>> = props.get("name").unwrap().as_str().unwrap().split(";").map(|s| {
            abbr.process(&String::from(s), &iso.to_ascii_uppercase())
        }).collect();

        let names = serde_json::to_value(names).unwrap();

        db.execute("
            UPDATE master
                SET
                    name = $2::JSONB
            WHERE
                id = $1
        ", &[&i, &names]).unwrap();
    });

    let new_max = new.max(&mut pool.get().unwrap()).unwrap();

    (1..=new_max).into_par_iter().for_each(|i| {
        let mut db = pool.get().unwrap();

        let val = match db.query("
            SELECT
                new.name,
                ST_AsGeoJSON(new.geom)::JSON AS geom,
                Array_To_Json((Array_Agg(
                    JSON_Build_Object(
                        'id', master.id,
                        'name', master.name,
                        'geom', master.geom
                    )
                    ORDER BY ST_Distance(master.geom, new.geom)
                ))[:10]) AS nets
            FROM
                master
                    INNER JOIN new
                        ON ST_DWithin(master.geom, new.geom, 0.001)
            WHERE
                new.id = $1
            GROUP BY
                new.id,
                new.name,
                new.geom
        ", &[&i]) {
            Err(err) => panic!("{}", err.to_string()),
            Ok(rows) => {
                let row = rows.get(0).unwrap();

                let name: serde_json::Value = row.get(0);
                let geom: serde_json::Value = row.get(1);
                let nets: serde_json::Value = row.get(2);
            }
        };
    });
}

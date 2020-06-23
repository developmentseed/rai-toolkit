use crate::pg::{Table, InputTable, Network};
use crate::{Tokens, Name, Names, Context, pg};
use crate::stream::{GeoStream, NetStream};
use crate::text::linker;
use rayon::prelude::*;
use crate::filter;
use std::thread;

#[derive(Serialize, Deserialize)]
pub struct DbSerial {
    id: i64,
    props: serde_json::Value,
    names: Vec<Name>,
    geom: serde_json::Value,
    cov: f64
}

pub struct DbType {
    id: i64,
    cov: f64,
    props: serde_json::Value,
    names: Names,
    geom: serde_json::Value
}

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();

    let langs: Vec<String> = args.value_of("langs").unwrap().to_string().to_lowercase().split(',').map(|i| {
        String::from(i.trim())
    }).collect();

    let output = args.value_of("output").unwrap().to_string();

    let context = Context::new(iso, None, Tokens::generate(langs));

    let master_src = args.value_of("MASTER").unwrap().to_string();
    let new_src = args.value_of("NEW").unwrap().to_string();

    let buffer: i64 = match args.value_of("BUFFER") {
        None => 25,
        Some(buffer) => match buffer.parse::<i64>() {
            Ok(buffer) => buffer,
            _ => panic!("--buffer value must be an integer")
        }
    };

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
            surface(&mut db, &master);
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
            surface(&mut db, &new);
            new.index(&mut db);
            new.seq(&mut db);
            println!("ok - imported {} new lines", new.count(&mut db));

        }));
    }

    for thread in manager {
        thread.join().unwrap();
    }

    name(&pool, &master, &context);
    name(&pool, &new, &context);

    let new_max = new.max(&mut pool.get().unwrap()).unwrap();

    (1..=new_max).into_par_iter().for_each(|i| {
        let mut db = pool.get().unwrap();

        match db.query(format!("
            SELECT
                new.props,
                new.name,
                ST_AsGeoJSON(new.geom)::JSON AS geom,
                ST_Length(new.geom) AS length,
                Array_To_Json((Array_Agg(
                    JSON_Build_Object(
                        'id', master.id,
                        'props', master.props,
                        'names', master.name,
                        'geom', master.geom,
                        'cov', ST_Length(ST_Intersection(
                            ST_Buffer(new.geom::GEOGRAPHY, {buffer})::GEOMETRY,
                            master.geom
                        ))
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
                new.props,
                new.geom
        ",
            buffer = &buffer
        ).as_str(), &[&i]) {
            Err(err) => panic!("{}", err.to_string()),
            Ok(rows) => {
                let row = match rows.get(0) {
                    Some(row) => row,
                    None => {
                        // Inner join failed to return any results - meaning new item
                        // does not have existing roads near it
                        db.execute("
                            INSERT INTO master (
                                name,
                                props,
                                geom
                            ) SELECT
                                name,
                                props,
                                geom
                            FROM
                                new
                            WHERE
                                id = $1
                        ", &[&i]).unwrap();
                        return ();
                    }
                };

                let props: serde_json::Value = row.get(0);
                let props = match props {
                    serde_json::Value::Object(props) => props,
                    _ => panic!("props must be an object")
                };

                let names: serde_json::Value = row.get(1);
                let names: Vec<Name> = serde_json::from_value(names).unwrap();
                let names = Names { names: names };

                let _geom: serde_json::Value = row.get(2);
                let length: f64 = row.get(3);
                let nets: Option<serde_json::Value> = row.get(4);

                if nets.is_none() || !props.contains_key("name") || props.get("name").unwrap().is_null() {
                    // For now, roads without names are automatically inserted into final db
                    // In the future a geometric comparison should be performed
                    db.execute("
                        INSERT INTO master (
                            name,
                            props,
                            geom
                        ) SELECT
                            name,
                            props,
                            geom
                        FROM
                            new
                        WHERE
                            id = $1
                    ", &[&i]).unwrap();
                    return ();
                } else {
                    let nets: Vec<DbSerial> = match serde_json::from_value(nets.unwrap()) {
                        Err(err) => panic!("JSON Failure: {}", err.to_string()),
                        Ok(nets) => nets
                    };

                    let mut pnets: Vec<DbType> = Vec::with_capacity(nets.len());
                    for net in nets {
                        pnets.push(DbType {
                            id: net.id,
                            geom: net.geom,
                            props: net.props,
                            cov: net.cov,
                            names: Names {
                                names: net.names
                            }
                        });
                    }

                    // The new road segment is too geometrically
                    // smiliar to import
                    if pnets.iter().any(|net| {
                        net.cov > length * 0.75
                    }) {
                        return ();
                    }

                    let primary = linker::Link::new(i, &names);
                    let potentials: Vec<linker::Link> = pnets.iter().map(|net| {
                        linker::Link::new(net.id, &net.names)
                    }).collect();

                    let props = serde_json::Value::from(props);
                    match linker::linker(primary, potentials, false) {
                        Some(link_match) => {
                            db.execute(r#"
                                UPDATE master
                                    SET
                                        props = props || $2 || '{ "conflated": true }'::JSONB
                                    WHERE
                                        id = $1
                            "#, &[&link_match.id, &props]).unwrap();
                        },
                        None => ()
                    };
                }
            }
        };
    });

    let mut output = std::fs::File::create(output).unwrap();

    let mut stream = pg::stream::PGStream::new(pool.get().unwrap(), String::from("next"), String::from("
        DECLARE next CURSOR FOR
            SELECT
                json_build_object(
                    'type', 'Feature',
                    'properties', props,
                    'geometry', ST_AsGeoJSON(geom)::JSON
                )::TEXT
            FROM
                master
    "), &[]).unwrap();

    std::io::copy(&mut stream, &mut output).unwrap();
}

fn name(pool: &r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, table: &(impl Table + std::marker::Sync), context: &Context) {
    let max = table.max(&mut pool.get().unwrap()).unwrap();

    (1..=max).into_par_iter().for_each(|i| {
        let mut db = pool.get().unwrap();

        let props = table.props(&mut db, i);

        if !props.contains_key("name") || props.get("name").unwrap().is_null() {
            db.execute(format!("
                UPDATE {table}
                    SET
                        name = '[]'::JSONB
                WHERE
                    id = $1
            ",
                table = table.name()
            ).as_str(), &[&i]).unwrap();
        } else {
            let names: Vec<Name> = props.get("name").unwrap().as_str().unwrap().split(";").map(|s| {
                Name::new(s, 0, None, &context)
            }).collect();

            let names = serde_json::to_value(names).unwrap();

            db.execute(format!("
                UPDATE {table}
                    SET
                        name = $2::JSONB
                WHERE
                    id = $1
            ",
                table = table.name()
            ).as_str(), &[&i, &names]).unwrap();
        }
    });
}

fn surface(db: &mut postgres::Client, table: &impl Table) {
    let rejects = filter::reject_surface();

    for reject in rejects {
        db.execute(format!("
            DELETE FROM
                {table}
            WHERE
                props->>'surface' = $1
        ",
            table = table.name()
        ).as_str(), &[&reject.to_string()]).unwrap();
    }
}

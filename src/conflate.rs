use crate::pg::{Table, InputTable, Network};
use crate::stream::{GeoStream, NetStream};
use rayon::prelude::*;
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();
    let iso = args.value_of("lang").unwrap().to_string().to_lowercase();

    let master_src = args.value_of("MASTER").unwrap().to_string();
    let new_src = args.value_of("NEW").unwrap().to_string();

    let master = Network::new("master");
    let new = Network::new("new");

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

            db.execute("
                ALTER TABLE master
                    ADD COLUMN name JSONB
            ", &[]).unwrap();

            db.execute("
                DROP SEQUENCE IF EXISTS master_seq;
            ", &[]).unwrap();

            db.execute("
                CREATE SEQUENCE master_seq;
            ", &[]).unwrap();

            db.execute("
                ALTER TABLE master
                    ALTER COLUMN id
                        SET DEFAULT nextval('master_seq');
            ", &[]).unwrap();

            db.execute("
                ALTER SEQUENCE master_seq OWNED BY master.id;
            ", &[]).unwrap();

            db.execute("
                UPDATE master
                    SET
                        id = nextval('master_seq')
            ", &[]).unwrap();
        }));
    }

    {
        let mut db = pool.get().unwrap();
        manager.push(thread::spawn(move || {
            new.create(&mut db);
            new.input(&mut db, NetStream::new(
                GeoStream::new(Some(new_src)),
                Some(String::from("/tmp/new_error.log")))
            );
            new.index(&mut db);
            println!("ok - imported {} new lines", new.count(&mut db));

            db.execute("
                ALTER TABLE new
                    ADD COLUMN name JSONB
            ", &[]).unwrap();
        }));
    }

    for thread in manager {
        thread.join().unwrap();
    }

    let max: i64 = match pool.get().unwrap().query("
        SELECT
            MAX(id)
        FROM
            master
    ", &[]) {
        Err(err) => panic!("{}", err),
        Ok(res) => res.get(0).unwrap().get(0)
    };

    (1..=max).into_par_iter().for_each(|i| {
        let mut db = pool.get().unwrap();

        let props: serde_json::Value = match db.query("
            SELECT
                props
            FROM
                master
            WHERE
                id = $1
        ", &[&i]) {
            Err(err) => panic!("{}", err),
            Ok(res) => res.get(0).unwrap().get(0)
        };

        let props = props.as_object().unwrap();

        if !props.contains_key("name") {
            return;
        }

        let name: Vec<String> = props.get("name").unwrap().as_str().unwrap().split(";").map(|s| {
            String::from(s)
        }).collect();

        println!("{:?}", props);
    });
}

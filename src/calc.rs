use crate::pg::{Table, InputTable, Network, Country};
use crate::stream::{GeoStream, NetStream};
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {

    let master_src = args.value_of("NETWORK").unwrap().to_string();
    let _pop_src = args.value_of("POP").unwrap().to_string();

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
}

use crate::pg::{Table, InputTable, Network};
use crate::stream::{GeoStream, NetStream};
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let master_src = args.value_of("MASTER").unwrap().to_string();
    let new_src = args.value_of("NEW").unwrap().to_string();

    let master = Network::new("master");
    let new = Network::new("new");

    println!("ok - formatted database");

    let mut manager = Vec::with_capacity(2);
    {
        let pool = pool.clone();
        manager.push(thread::spawn(move || {
            let mut db = pool.get().unwrap();
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
        let pool = pool.clone();
        manager.push(thread::spawn(move || {
            let mut db = pool.get().unwrap();
            new.create(&mut db);
            new.input(&mut db, NetStream::new(
                GeoStream::new(Some(new_src)),
                Some(String::from("/tmp/new_error.log")))
            );
            new.index(&mut db);
            println!("ok - imported {} new lines", new.count(&mut db));
        }));
    }

    for thread in manager {
        thread.join().unwrap();
    }
}

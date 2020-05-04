use crate::pg::{Table, InputTable, Network};
use crate::stream::{GeoStream, NetStream};
use std::thread;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {

    let master_src = args.value_of("MASTER").unwrap().to_string();
    let _pop_src = args.value_of("POP").unwrap().to_string();

    let master = Network::new("master");
    println!("ok - formatted database");

    thread::spawn(move || {
        let mut db = pool.get().unwrap();
        master.create(&mut db);
        master.input(&mut db, NetStream::new(
                GeoStream::new(Some(master_src)),
                Some(String::from("/tmp/master_error.log")))
                    );
        master.index(&mut db);
        println!("ok - imported {} master lines", master.count(&mut db));
    });
}

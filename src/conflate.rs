use crate::pg::{Table, InputTable, Network};
use crate::stream::{GeoStream, NetStream};

pub fn main(mut db: postgres::Client, args: &clap_v3::ArgMatches) {
    println!("ok - formatted database");

    let master_src = args.value_of("MASTER").unwrap().to_string();

    let master = Network::new("master");
    master.create(&mut db);
    master.input(&mut db, NetStream::new(
        GeoStream::new(Some(master_src)),
        Some(String::from("/tmp/error.log")))
    );
    println!("ok - imported {} master lines", master.count(&mut db));

    let _new_src = args.value_of("NEW").unwrap();


}

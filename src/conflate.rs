pub fn main(mut db: postgres::Client, args: &clap_v3::ArgMatches) {
    tables(&mut db);
    println!("ok - formatted database");

    let master_src = args.value_of("INPUT").unwrap();
    let new_src = args.value_of("NEW").unwrap();

}

fn tables(db: &mut postgres::Client) {
    db.execute("
        DROP TABLE IF EXISTS master;
    ", &[]).unwrap();

    db.execute("
        DROP TABLE IF EXISTS new;
    ", &[]).unwrap();

    db.execute("
        CREATE UNLOGGED TABLE master (
            id      BIGINT
            props   JSONB
            geom    GEOMETRY(MULTILINESTRING, 4326)
        )
    ", &[]).unwrap();

    db.execute("
        CREATE UNLOGGED TABLE master (
            id      BIGINT
            props   JSONB
            geom    GEOMETRY(MULTILINESTRING, 4326)
        )
    ", &[]).unwrap();
}

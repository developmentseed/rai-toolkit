use crate::list::list;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();

    pool.get().unwrap().execute(format!("
        DROP SCHEMA country_{} CASCADE;
    ", iso).as_str(), &[]).unwrap();

    println!("\nCountry Dropped\n");
}

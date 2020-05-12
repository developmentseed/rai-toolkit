
pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, _args: &clap_v3::ArgMatches) {
    let countries = list(pool);

    if countries.len() == 0 {
        println!("\nNo Countries Loaded\n");
    } else {
        println!("\nLoaded Countries:");
        for country in countries {
            println!("- {}", country);
        }
        println!("");
    }
}

pub fn list(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>) -> Vec<String> {
    let mut db = pool.get().unwrap();

    match db.query("
        SELECT
            schema_name
        FROM
            information_schema.schemata
    ", &[]) {
        Err(err) => panic!("{}", err),
        Ok(rows) => {
            let mut countries = Vec::with_capacity(rows.len());
            for row in rows.iter() {
                let name: &str = row.get(0);
                if name.contains("country_") {
                    countries.push(String::from(name));
                }
            }

            countries
        }
    }
}

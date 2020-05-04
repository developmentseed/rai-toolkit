use clap_v3::{App, load_yaml};
use r2d2_postgres::PostgresConnectionManager;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let args = App::from(cli_cnf).get_matches();

    let db_str = args.value_of("database").unwrap_or("postgres://postgres@localhost:5432/rai");

    let manager = PostgresConnectionManager::new(
        db_str.parse().unwrap(),
        postgres::NoTls,
    );

    let pool = r2d2::Pool::new(manager).unwrap();

    match args.subcommand() {
        ("conflate", Some(sub_args)) => rai_toolkit::conflate::main(pool, sub_args),
        ("calc", Some(sub_args)) => rai_toolkit::calc::main(pool, sub_args),
        _ => {
            println!("Invalid Subcommand: ./rai-toolkit --help for valid options");
            std::process::exit(1);
        },
    }
}

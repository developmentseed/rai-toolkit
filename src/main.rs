use clap_v3::{App, load_yaml};
use postgres::{Client, NoTls};

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let args = App::from(cli_cnf).get_matches();

    let database = String::from(args.value_of("database").unwrap_or("postgres://postgres@localhost:5432/rai"));

    let client = match Client::connect(&database, NoTls) {
        Ok(client) => client,
        Err(err) => {
            println!();
            println!("Failed to connecto database: {}", &database);
            println!("  {}", err);
            println!();
            std::process::exit(1);
        },
    };

    match args.subcommand() {
        ("conflate", Some(sub_args)) => rai_toolkit::conflate::main(client, sub_args),
        ("calc", Some(sub_args)) => rai_toolkit::calc::main(client, sub_args),
        _ => {
            println!("Invalid Subcommand: ./rai-toolkit --help for valid options");
            std::process::exit(1);
        },
    }
}

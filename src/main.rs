use clap_v3::{App, load_yaml};

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let args = App::from(cli_cnf).get_matches();

    let database = String::from(args.value_of("database").unwrap_or("postgres@localhost:5432/rai"));

    match args.subcommand() {
        ("conflate", Some(sub_args)) => rai_toolkit::conflate::main(database, sub_args),
        ("calc", Some(sub_args)) => rai_toolkit::calc::main(database, sub_args),
        _ => {
            println!("Invalid Subcommand: ./rai-toolkit --help for valid options");
            std::process::exit(1);
        },
    }
}

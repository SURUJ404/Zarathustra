mod ops;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("astra")
        .version("0.1.0")
        .about("zkSNARK toolkit")
        .subcommand(SubCommand::with_name("prove").about("zkSNARK proof generation pipeline")
            .subcommand(SubCommand::with_name("compile").about("Compile a circuit source file into R1CS")
                .arg(clap::Arg::with_name("input").required(true).help("Source file"))
                .arg(clap::Arg::with_name("public").short("p").long("public").takes_value(true).default_value("").allow_hyphen_values(true).help("Public inputs (comma-separated)"))
                .arg(clap::Arg::with_name("private").short("r").long("private").takes_value(true).default_value("").allow_hyphen_values(true).help("Private inputs (comma-separated)")))
            .subcommand(SubCommand::with_name("setup").about("Generate CRS for a compiled circuit")
                .arg(clap::Arg::with_name("input").required(true).help("Source file"))
                .arg(clap::Arg::with_name("public").short("p").long("public").takes_value(true).default_value("").allow_hyphen_values(true).help("Public inputs (comma-separated)"))
                .arg(clap::Arg::with_name("private").short("r").long("private").takes_value(true).default_value("").allow_hyphen_values(true).help("Private inputs (comma-separated)")))
            .subcommand(SubCommand::with_name("prove").about("Generate a zkSNARK proof for a circuit")
                .arg(clap::Arg::with_name("input").required(true).help("Source file"))
                .arg(clap::Arg::with_name("public").short("p").long("public").takes_value(true).default_value("").allow_hyphen_values(true).help("Public inputs (comma-separated)"))
                .arg(clap::Arg::with_name("private").short("r").long("private").takes_value(true).default_value("").allow_hyphen_values(true).help("Private inputs (comma-separated)")))
            .subcommand(SubCommand::with_name("verify").about("Verify a proof against public inputs")
                .arg(clap::Arg::with_name("input").required(true).help("Source file"))
                .arg(clap::Arg::with_name("public").short("p").long("public").takes_value(true).default_value("").allow_hyphen_values(true).help("Public inputs (comma-separated)"))
                .arg(clap::Arg::with_name("private").short("r").long("private").takes_value(true).default_value("").allow_hyphen_values(true).help("Private inputs (comma-separated)"))))
        .subcommand(SubCommand::with_name("audit").about("Security audit for circuits and smart contracts")
            .arg(clap::Arg::with_name("target").help("Target file or directory to scan").default_value("."))
            .arg(clap::Arg::with_name("verbose").short("v").long("verbose").help("Verbose output"))
            .arg(clap::Arg::with_name("format").short("f").long("format").takes_value(true).default_value("terminal").possible_values(&["terminal", "json", "html"]).help("Output format"))
            .arg(clap::Arg::with_name("output").short("o").long("output").takes_value(true).help("Output file path"))
            .arg(clap::Arg::with_name("deny").long("deny").takes_value(true).possible_values(&["critical", "high", "medium", "any"]).help("Exit non-zero if findings at this severity or above exist")))
        .subcommand(SubCommand::with_name("deploy").about("Deploy verifier contracts and manage on-chain verification")
            .subcommand(SubCommand::with_name("init").about("Initialize a new circuit project")
                .arg(clap::Arg::with_name("name").required(true).help("Project name")))
            .subcommand(SubCommand::with_name("verifier").about("Export a Solidity verifier contract")
                .arg(clap::Arg::with_name("output").short("o").long("output").takes_value(true).default_value("Verifier.sol").help("Output file")))
            .subcommand(SubCommand::with_name("verify").about("Verify a proof on-chain (simulated)")))
        .get_matches();

    match matches.subcommand() {
        ("prove", Some(m)) => ops::prove::exec(m),
        ("audit", Some(m)) => ops::audit::exec(m),
        ("deploy", Some(m)) => ops::deploy::exec(m),
        _ => {
            println!("astra 0.1.0 - zkSNARK toolkit");
            println!("Subcommands: prove, audit, deploy");
            println!("Use 'astra <cmd> --help' for details");
        }
    }
}

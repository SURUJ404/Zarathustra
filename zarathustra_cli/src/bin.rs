#[macro_use]
extern crate lazy_static;

mod cli_constants;
mod ops;

use clap::{App, AppSettings, Arg, SubCommand};
use ops::*;

fn main() {
    std::panic::set_hook(Box::new(panic_hook));
    env_logger::init();
    cli().unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    })
}

fn cli() -> Result<(), String> {
    let matches = App::new("Zarathustra")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("zkSNARK proving, circuit auditing, and smart contract deployment toolkit")
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Verbose mode")
                .required(false)
                .global(true),
        )
        .subcommands(vec![
            SubCommand::with_name("prove")
                .about("zkSNARK proof generation pipeline: compile, setup, witness, prove, verify")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommands(vec![
                    compile::subcommand(),
                    inspect::subcommand(),
                    check::subcommand(),
                    compute_witness::subcommand(),
                    #[cfg(feature = "ark")]
                    universal_setup::subcommand(),
                    #[cfg(feature = "bellman")]
                    mpc::subcommand(),
                    #[cfg(feature = "bellperson")]
                    nova::subcommand(),
                    #[cfg(any(feature = "bellman", feature = "ark"))]
                    setup::subcommand(),
                    export_verifier::subcommand(),
                    #[cfg(any(feature = "bellman", feature = "ark"))]
                    generate_proof::subcommand(),
                    generate_smtlib2::subcommand(),
                    print_proof::subcommand(),
                    #[cfg(any(feature = "bellman", feature = "ark"))]
                    verify::subcommand(),
                    profile::subcommand(),
                ]),
            audit::subcommand(),
            deploy::subcommand(),
        ])
        .get_matches();

    match matches.subcommand() {
        ("prove", Some(sub_matches)) => match sub_matches.subcommand() {
            ("compile", Some(sub)) => compile::exec(sub),
            ("inspect", Some(sub)) => inspect::exec(sub),
            ("check", Some(sub)) => check::exec(sub),
            ("compute-witness", Some(sub)) => compute_witness::exec(sub),
            #[cfg(feature = "ark")]
            ("universal-setup", Some(sub)) => universal_setup::exec(sub),
            #[cfg(feature = "bellman")]
            ("mpc", Some(sub)) => mpc::exec(sub),
            #[cfg(feature = "bellperson")]
            ("nova", Some(sub)) => nova::exec(sub),
            #[cfg(any(feature = "bellman", feature = "ark"))]
            ("setup", Some(sub)) => setup::exec(sub),
            ("export-verifier", Some(sub)) => export_verifier::exec(sub),
            #[cfg(any(feature = "bellman", feature = "ark"))]
            ("generate-proof", Some(sub)) => generate_proof::exec(sub),
            ("generate-smtlib2", Some(sub)) => generate_smtlib2::exec(sub),
            ("print-proof", Some(sub)) => print_proof::exec(sub),
            #[cfg(any(feature = "bellman", feature = "ark"))]
            ("verify", Some(sub)) => verify::exec(sub),
            ("profile", Some(sub)) => profile::exec(sub),
            _ => unreachable!(),
        },
        ("audit", Some(sub_matches)) => audit::exec(sub_matches),
        ("deploy", Some(sub_matches)) => deploy::exec(sub_matches),
        _ => unreachable!(),
    }
}

fn panic_hook(pi: &std::panic::PanicHookInfo) {
    println!("The compiler unexpectedly panicked");
    println!("{}", pi);
    #[cfg(debug_assertions)]
    {
        use std::backtrace::{Backtrace, BacktraceStatus};
        let backtrace = Backtrace::capture();
        if backtrace.status() == BacktraceStatus::Captured {
            println!("rust backtrace:\n{}", backtrace);
        }
    }
    println!(
        "This is unexpected, please submit a full bug report at https://github.com/SURUJ404/Zarathustra/issues"
    );
}

#[cfg(test)]
mod tests {
    extern crate glob;
    use self::glob::glob;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::string::String;
    use typed_arena::Arena;
    use zarathustra_common::CompileConfig;
    use zarathustra_core::compile::{compile, CompilationArtifacts};
    use zarathustra_field::Bn128Field;
    use zarathustra_fs_resolver::FileSystemResolver;

    #[test]
    fn compile_examples() {
        let builder = std::thread::Builder::new().stack_size(8388608);
        builder
            .spawn(|| {
                for p in glob("./examples/**/*").expect("Failed to read glob pattern") {
                    let path = match p {
                        Ok(x) => x,
                        Err(why) => panic!("Error: {:?}", why),
                    };
                    if !path.is_file() {
                        continue;
                    }
                    let extension = path.extension();
                    if ["", "sh"].contains(
                        &extension
                            .map(|e| e.to_str().unwrap_or_default())
                            .unwrap_or_default(),
                    ) {
                        continue;
                    }
                    println!("Testing {:?}", path);
                    assert_eq!(path.extension().expect("extension expected"), "zara");
                    let should_error = path.to_str().unwrap().contains("compile_errors");
                    let file = File::open(path.clone()).unwrap();
                    let mut reader = BufReader::new(file);
                    let mut source = String::new();
                    reader.read_to_string(&mut source).unwrap();
                    let stdlib =
                        std::fs::canonicalize("../zarathustra_stdlib/stdlib").unwrap();
                    let resolver =
                        FileSystemResolver::with_stdlib_root(stdlib.to_str().unwrap());
                    let arena = Arena::new();
                    let res = compile::<Bn128Field, _>(
                        source,
                        path,
                        Some(&resolver),
                        CompileConfig::default(),
                        &arena,
                    );
                    assert_eq!(res.is_err(), should_error);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn execute_examples_ok() {
        for p in glob("./examples/test*").expect("Failed to read glob pattern") {
            let path = match p {
                Ok(x) => x,
                Err(why) => panic!("Error: {:?}", why),
            };
            println!("Testing {:?}", path);
            assert_eq!(path.extension().expect("extension expected"), "zara");
            let file = File::open(path.clone()).unwrap();
            let mut reader = BufReader::new(file);
            let mut source = String::new();
            reader.read_to_string(&mut source).unwrap();
            let stdlib = std::fs::canonicalize("../zarathustra_stdlib/stdlib").unwrap();
            let resolver = FileSystemResolver::with_stdlib_root(stdlib.to_str().unwrap());
            let arena = Arena::new();
            let artifacts: CompilationArtifacts<Bn128Field, _> = compile(
                source,
                path,
                Some(&resolver),
                CompileConfig::default(),
                &arena,
            )
            .unwrap();
            let interpreter = zarathustra_interpreter::Interpreter::default();
            let prog = artifacts.prog();
            let _ = interpreter
                .execute(
                    &[Bn128Field::from(0u32)],
                    prog.statements.into_iter(),
                    &prog.arguments,
                    &prog.solvers,
                )
                .unwrap();
        }
    }

    #[test]
    fn execute_examples_err() {
        for p in glob("./examples/runtime_errors/*").expect("Failed to read glob pattern") {
            let path = match p {
                Ok(x) => x,
                Err(why) => panic!("Error: {:?}", why),
            };
            println!("Testing {:?}", path);
            assert_eq!(path.extension().expect("extension expected"), "zara");
            let file = File::open(path.clone()).unwrap();
            let mut reader = BufReader::new(file);
            let mut source = String::new();
            reader.read_to_string(&mut source).unwrap();
            let stdlib = std::fs::canonicalize("../zarathustra_stdlib/stdlib").unwrap();
            let resolver = FileSystemResolver::with_stdlib_root(stdlib.to_str().unwrap());
            let arena = Arena::new();
            let artifacts: CompilationArtifacts<Bn128Field, _> = compile(
                source,
                path,
                Some(&resolver),
                CompileConfig::default(),
                &arena,
            )
            .unwrap();
            let interpreter = zarathustra_interpreter::Interpreter::default();
            let prog = artifacts.prog();
            let res = interpreter.execute(
                &[Bn128Field::from(0)],
                prog.statements.into_iter(),
                &prog.arguments,
                &prog.solvers,
            );
            assert!(res.is_err());
        }
    }
}

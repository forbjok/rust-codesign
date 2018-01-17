extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use] extern crate log;

extern crate codesign;

use log::LevelFilter;

use codesign::{SignParams, SignTool, CodeSignError};

fn main() {
    use std::process;

    use clap::{App, Arg};

    let matches = App::new("codesign")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("file")
                .help("Files to sign")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("digest-algorithm")
                .short("d")
                .takes_value(true)
                .default_value("sha256")
                .required(true)
                .help("Specify digest algorithm"),
        )
        .arg(
            Arg::with_name("certificate-thumbprint")
                .short("c")
                .takes_value(true)
                .required(true)
                .help("Specify certificate thumbprint (SHA1)"),
        )
        .arg(
            Arg::with_name("timestamp-url")
                .short("t")
                .takes_value(true)
                .help("Specify timestamp URL"),
        )
        .get_matches();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Off,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 | _ => LevelFilter::Trace,
    };

    initialize_logging(log_level);

    debug!("Debug logging enabled.");

    let files = matches.values_of("file").expect("No files specified!").into_iter();
    let digest_algorithm: &str = matches.value_of("digest-algorithm").unwrap();
    let certificate_thumbprint: &str = matches.value_of("certificate-thumbprint").unwrap();
    let timestamp_url: Option<&str> = matches.value_of("timestamp-url");

    // Locate latest SignTool
    let signtool = match SignTool::locate_latest() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Couldn't locate SignTool: {}", err.to_string());
            process::exit(2)
        }
    };

    // Set up signing parameters
    let sign_params = SignParams {
        digest_algorithm: digest_algorithm.to_owned(),
        certificate_thumbprint: certificate_thumbprint.to_owned(),
        timestamp_url: match timestamp_url {
            Some(v) => Some(v.to_owned()),
            None => None
        },
    };

    let mut error_count: i32 = 0;
    let mut last_signtool_error_exit_code: i32 = 0;

    // Sign specified files
    for file in files {
        eprint!("Signing {}... ", file);

        match signtool.sign(file, &sign_params) {
            Ok(()) => eprintln!("OK."),
            Err(err) => {
                error_count += 1;
                eprintln!("{}", err.to_string());

                /* If it's a SignTool error, set last SignTool error exit code. */
                if let CodeSignError::SignToolError { exit_code, .. } = err {
                    last_signtool_error_exit_code = exit_code;
                }
            }
        };
    }

    if error_count > 0 {
        // If there were errors, return a non-zero exit code
        process::exit(last_signtool_error_exit_code);
    }
}

fn initialize_logging(our_level_filter: LevelFilter) {
    use chrono::Utc;
    use log::LevelFilter;

    fern::Dispatch::new()
        .level(LevelFilter::Off)
        .level_for(env!("CARGO_PKG_NAME"), our_level_filter)
        .chain(std::io::stderr())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} | {} | {} | {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();
}

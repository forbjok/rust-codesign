extern crate chrono;
extern crate clap;
extern crate fern;
#[macro_use] extern crate log;

extern crate codesign;

use log::LevelFilter;

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
                .required(true)
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
    let timestamp_url: &str = matches.value_of("timestamp-url").unwrap();

    // Locate latest SignTool
    let signtool = codesign::SignTool::locate_latest().expect("Error locating SignTool!");

    // Set up signing parameters
    let sign_params = codesign::SignParams {
        digest_algorithm: digest_algorithm.to_owned(),
        certificate_thumbprint: certificate_thumbprint.to_owned(),
        timestamp_url: timestamp_url.to_owned(),
    };

    let mut error_count: i32 = 0;

    // Sign specified files
    for file in files {
        print!("Signing {}... ", file);

        match signtool.sign(file, &sign_params) {
            Ok(()) => println!("OK."),
            Err(_) => {
                error_count += 1;

                println!("Error!");
            }
        };
    }

    if error_count > 0 {
        // If there were errors, return a non-zero exit code
        process::exit(101);
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

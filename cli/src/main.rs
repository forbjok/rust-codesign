use log::{debug, LevelFilter};
use structopt::StructOpt;

use codesign::{SignParams, SignTool, CodeSignError};

#[derive(Debug, StructOpt)]
#[structopt(name = "CodeSign", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences), help = "Verbosity")]
    verbosity: u8,
    #[structopt(name = "file", help = "Files to sign")]
    files: Vec<String>,
    #[structopt(name = "digest-algorithm", short = "d", help = "Specify digest algorithm", default_value = "sha256")]
    digest_algorithm: String,
    #[structopt(name = "certificate-thumbprint", short = "c", help = "Specify certificate thumbprint (SHA1)")]
    certificate_thumbprint: String,
    #[structopt(name = "timestamp-url", short = "t", help = "Specify timestamp URL")]
    timestamp_url: Option<String>,
}

fn main() {
    use std::process;

    let opt = Opt::from_args();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let log_level = match opt.verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 | _ => LevelFilter::Trace,
    };

    // Initialize logging
    initialize_logging(log_level);

    debug!("Debug logging enabled.");

    let files = &opt.files;
    let digest_algorithm = &opt.digest_algorithm;
    let certificate_thumbprint = &opt.certificate_thumbprint;
    let timestamp_url = &opt.timestamp_url;

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

    const BIN_MODULE: &str = env!("CARGO_CRATE_NAME");
    const LIB_MODULE: &str = "codesign";

    fern::Dispatch::new()
        .level(LevelFilter::Error)
        .level_for(BIN_MODULE, our_level_filter)
        .level_for(LIB_MODULE, our_level_filter)
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

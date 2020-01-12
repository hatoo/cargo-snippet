mod config;
mod fsutil;
mod parser;
mod snippet;
mod writer;

use std::fs;
use std::io::Read;

use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};
use log::error;

use std::error::Error;

/// Report error and continue.
fn report_error<T, E: Error>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(x) => Some(x),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn main() {
    env_logger::init();

    // Setup for cargo subcommand
    let matches = App::new("cargo-snippet")
        .version(crate_version!())
        .bin_name("cargo")
        .settings(&[AppSettings::GlobalVersion, AppSettings::SubcommandRequired])
        .subcommand(
            SubCommand::with_name("snippet")
                .author(crate_authors!())
                .about("Extract code snippet from cargo projects")
                .arg(Arg::with_name("PATH").multiple(true).help(
                    "The files or directories (including children) \
                     to extract snippet (defaults to <project_root>/src when omitted)",
                ))
                .arg(
                    Arg::with_name("output_type")
                        .long("type")
                        .short("t")
                        .default_value("neosnippet")
                        .possible_values(&["neosnippet", "vscode", "ultisnips"]),
                ),
        )
        .get_matches();

    let config = config::Config::from_matches(&matches);

    // Alphabetical order
    let mut snippets = Vec::new();

    let mut buf = String::new();
    for path in config.target.iter_paths() {
        buf.clear();
        log::info!("Start read {:?}", &path);
        if let Some(mut file) = report_error(fs::File::open(path)) {
            if report_error(file.read_to_string(&mut buf)).is_some() {
                if let Some(mut parsed) = report_error(parser::parse_snippet(&buf)) {
                    snippets.append(&mut parsed);
                }
            }
        }
    }

    config
        .output_type
        .write(&snippet::process_snippets(&snippets));
}

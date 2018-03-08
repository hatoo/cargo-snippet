#[macro_use]
extern crate clap;
extern crate glob;
#[cfg(test)]
#[macro_use]
extern crate quote;
#[cfg(not(test))]
extern crate quote;
extern crate rustfmt_nightly;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate syn;

mod parser;
mod fsutil;
mod config;
mod writer;

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;

use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
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
    let mut snippets = BTreeMap::new();

    let mut buf = String::new();
    for path in config.target.iter_paths() {
        buf.clear();
        if let Ok(mut file) = fs::File::open(path) {
            if file.read_to_string(&mut buf).is_ok() {
                for (name, content) in parser::parse_snippet(&buf) {
                    *snippets.entry(name).or_insert(String::new()) += &content;
                }
            }
        }
    }

    config.output_type.write(&snippets);
}

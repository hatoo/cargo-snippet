extern crate glob;
extern crate quote;
extern crate rustfmt_nightly;
extern crate syn;

mod parser;
mod fsutil;

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;

use rustfmt_nightly::config::Config;

fn main() {
    // Alphabetical order
    let mut snippets = BTreeMap::new();

    if let Some(mut path) = fsutil::project_root_path() {
        path.push("src");
        path.push("**");
        path.push("*.rs");

        let mut buf = String::new();
        for path in glob::glob(&format!("{}", path.display()))
            .unwrap()
            .filter_map(Result::ok)
        {
            buf.clear();
            if let Ok(mut file) = fs::File::open(path) {
                if file.read_to_string(&mut buf).is_ok() {
                    for (name, content) in parser::parse_snippet(&buf) {
                        *snippets.entry(name).or_insert(String::new()) += &content;
                    }
                }
            }
        }
    }

    let config = Config::default();
    for (name, content) in snippets.into_iter() {
        if let Some(formatted) = rustfmt_nightly::format_snippet(&content, &config) {
            println!("snippet {}", name);
            for line in formatted.lines() {
                println!("    {}", line);
            }
            println!();
        }
    }
}

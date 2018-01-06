use clap::ArgMatches;
use std::path::{Path, PathBuf};
use std::iter;
use std::fs;

use glob::glob;
use fsutil;

#[derive(Debug)]
pub enum Config<'a> {
    // <project_root>/src. Default
    ProjectSrc,
    // Args
    Paths(Vec<&'a str>),
}

impl<'a> Config<'a> {
    pub fn from_matches(matches: &'a ArgMatches) -> Self {
        matches
            .subcommand_matches("snippet")
            .and_then(|m| {
                m.values_of("PATH")
                    .map(|path| Config::Paths(path.collect()))
            })
            .unwrap_or(Config::ProjectSrc)
    }

    pub fn iter_paths(&self) -> Box<Iterator<Item = PathBuf> + 'a> {
        match self {
            &Config::ProjectSrc => fsutil::project_root_path()
                .and_then(|mut path| {
                    path.push("src");
                    path.push("**");
                    path.push("*.rs");
                    glob(&format!("{}", path.display())).ok().map(|paths| {
                        Box::new(paths.filter_map(|e| e.ok())) as Box<Iterator<Item = PathBuf>>
                    })
                })
                .unwrap_or(Box::new(iter::empty())),
            &Config::Paths(ref v) => Box::new(
                v.clone()
                    .into_iter()
                    .filter_map(|s| {
                        fs::metadata(Path::new(s)).ok().and_then(|meta| {
                            let path = if meta.is_dir() {
                                let mut path = Path::new(s).to_path_buf();
                                path.push("**");
                                path.push("*.rs");
                                path
                            } else {
                                Path::new(s).to_path_buf()
                            };
                            glob(&format!("{}", path.display()))
                                .ok()
                                .map(|paths| paths.filter_map(|e| e.ok()))
                        })
                    })
                    .flat_map(|i| i),
            ),
        }
    }
}

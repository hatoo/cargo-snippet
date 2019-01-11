use clap::ArgMatches;
use std::collections::BTreeMap;
use std::fs;
use std::iter;
use std::path::{Path, PathBuf};

use crate::fsutil;
use crate::writer;
use glob::glob;

#[derive(Debug)]
pub struct Config<'a> {
    pub target: Target<'a>,
    pub output_type: OutputType,
}

#[derive(Debug)]
pub enum Target<'a> {
    // <project_root>/src. Default
    ProjectSrc,
    // Args
    Paths(Vec<&'a str>),
}

#[derive(Debug)]
pub enum OutputType {
    Neosnippet,
    VScode,
    Ultisnips,
}

impl<'a> Config<'a> {
    pub fn from_matches(matches: &'a ArgMatches) -> Self {
        Config {
            target: Target::from_matches(matches),
            output_type: OutputType::from_matches(matches),
        }
    }
}

impl<'a> Target<'a> {
    fn from_matches(matches: &'a ArgMatches) -> Self {
        matches
            .subcommand_matches("snippet")
            .and_then(|m| {
                m.values_of("PATH")
                    .map(|path| Target::Paths(path.collect()))
            })
            .unwrap_or(Target::ProjectSrc)
    }

    pub fn iter_paths(&self) -> Box<Iterator<Item = PathBuf> + 'a> {
        match self {
            Target::ProjectSrc => fsutil::project_root_path()
                .and_then(|mut path| {
                    path.push("src");
                    path.push("**");
                    path.push("*.rs");
                    glob(&format!("{}", path.display())).ok().map(|paths| {
                        Box::new(paths.filter_map(|e| e.ok())) as Box<Iterator<Item = PathBuf>>
                    })
                })
                .unwrap_or_else(|| Box::new(iter::empty())),
            Target::Paths(ref v) => Box::new(
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

impl OutputType {
    fn from_matches(matches: &ArgMatches) -> Self {
        matches
            .subcommand_matches("snippet")
            .and_then(|m| {
                m.value_of("output_type").map(|t| match t {
                    "vscode" => OutputType::VScode,
                    "ultisnips" => OutputType::Ultisnips,
                    _ => OutputType::Neosnippet,
                })
            })
            .unwrap_or(OutputType::Neosnippet)
    }

    pub fn write(&self, snippets: &BTreeMap<String, String>) {
        match self {
            OutputType::Neosnippet => {
                writer::write_neosnippet(snippets);
            }
            OutputType::VScode => {
                writer::write_vscode(snippets);
            }
            OutputType::Ultisnips => {
                writer::write_ultisnips(snippets);
            }
        }
    }
}

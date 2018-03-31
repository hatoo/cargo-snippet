use rustfmt_nightly;
use serde_json;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct VScode {
    prefix: String,
    body: Vec<String>,
}

pub fn format_src(src: &str) -> Option<String> {
    let mut rustfmt_config = rustfmt_nightly::config::Config::default();
    rustfmt_config
        .set()
        .write_mode(rustfmt_nightly::config::WriteMode::Plain);

    let mut out = Vec::with_capacity(src.len() * 2);
    let input = rustfmt_nightly::Input::Text(src.into());
    rustfmt_nightly::format_input(input, &rustfmt_config, Some(&mut out))
        .ok()
        .and_then(|_| String::from_utf8(out).ok())
}

pub fn write_neosnippet(snippets: &BTreeMap<String, String>) {
    for (name, content) in snippets.iter() {
        if let Some(formatted) = format_src(content) {
            println!("snippet {}", name);
            for line in formatted.lines() {
                println!("    {}", line);
            }
            println!();
        }
    }
}

pub fn write_vscode(snippets: &BTreeMap<String, String>) {
    let vscode: BTreeMap<String, VScode> = snippets
        .iter()
        .filter_map(|(name, content)| {
            format_src(content).map(|formatted| {
                (
                    name.to_owned(),
                    VScode {
                        prefix: name.to_owned(),
                        body: formatted.lines().map(|l| l.to_owned()).collect(),
                    },
                )
            })
        })
        .collect();

    if let Ok(json) = serde_json::to_string_pretty(&vscode) {
        println!("{}", json);
    }
}

pub fn write_ultisnips(snippets: &BTreeMap<String, String>) {
    for (name, content) in snippets.iter() {
        if let Some(formatted) = format_src(content) {
            println!("snippet {}", name);
            print!("{}", formatted);
            println!("endsnippet");
            println!();
        }
    }
}

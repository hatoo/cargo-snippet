use rustfmt_nightly;
use serde_json;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct VScode {
    prefix: String,
    body: Vec<String>,
}

pub fn write_neosnippet(snippets: &BTreeMap<String, String>) {
    let rustfmt_config = rustfmt_nightly::Config::default();

    for (name, content) in snippets.iter() {
        if let Some(formatted) = rustfmt_nightly::format_snippet(content, &rustfmt_config) {
            println!("snippet {}", name);
            for line in formatted.lines() {
                println!("    {}", line);
            }
            println!();
        }
    }
}

pub fn write_vscode(snippets: &BTreeMap<String, String>) {
    let rustfmt_config = rustfmt_nightly::Config::default();

    let vscode: BTreeMap<String, VScode> = snippets
        .iter()
        .filter_map(|(name, content)| {
            rustfmt_nightly::format_snippet(content, &rustfmt_config).map(|formatted| {
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

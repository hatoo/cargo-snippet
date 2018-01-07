use rustfmt_nightly;
use std::collections::BTreeMap;

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

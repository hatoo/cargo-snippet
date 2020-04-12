use serde_derive::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct VScode {
    prefix: String,
    body: Vec<String>,
}

#[cfg(feature = "inner_rustfmt")]
pub fn format_src(src: &str) -> Option<String> {
    let mut rustfmt_config = rustfmt_nightly::Config::default();
    rustfmt_config
        .set()
        .emit_mode(rustfmt_nightly::EmitMode::Stdout);
    rustfmt_config
        .set()
        .verbose(rustfmt_nightly::Verbosity::Quiet);

    let mut out = Vec::with_capacity(src.len() * 2);
    let input = rustfmt_nightly::Input::Text(src.into());

    if rustfmt_nightly::Session::new(rustfmt_config, Some(&mut out))
        .format(input)
        .is_ok()
    {
        String::from_utf8(out).ok().map(|s| s.replace("\r\n", "\n"))
    } else {
        None
    }
}

#[cfg(not(feature = "inner_rustfmt"))]
pub fn format_src(src: &str) -> Option<String> {
    use std::io::Write;
    use std::process;

    let mut command = process::Command::new("rustfmt")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt process");
    {
        let mut stdin = command.stdin.take()?;
        write!(stdin, "{}", src).unwrap();
    }
    let out = command.wait_with_output().ok()?;

    if !out.status.success() {
        log::error!("rustfmt returns non-zero status");
        return None;
    }

    let stdout = out.stdout;
    let out = String::from_utf8(stdout).ok()?;
    Some(out.replace("\r\n", "\n"))
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
                        body: formatted
                            .lines()
                            .map(|l|
                                // Escape "$" to disable placeholder
                                l.to_owned().replace("$", "\\$"))
                            .collect(),
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

#[test]
fn test_format_src() {
    assert_eq!(format_src("fn foo(){}"), Some("fn foo() {}\n".into()));

    assert_eq!(
        format_src("/// doc comment\n pub fn foo(){}"),
        Some("/// doc comment\npub fn foo() {}\n".into())
    );
}

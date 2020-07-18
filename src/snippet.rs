use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Debug)]
pub struct SnippetAttributes {
    // A snippet with multiple names is allowed but using dependency is recommended.
    pub names: HashSet<String>,
    // Dependencies
    pub uses: HashSet<String>,
    // Prefix for snippet. It's will be emitted prior to the snippet.
    pub prefix: String,
    // Whether doc comments associated with this snippet should be hidden or not.
    pub doc_hidden: bool,
}

#[derive(Debug)]
pub struct Snippet {
    pub attrs: SnippetAttributes,
    // Snippet content (Not formated)
    pub content: String,
}

pub fn process_snippets(snips: &[Snippet]) -> BTreeMap<String, String> {
    #[derive(Default, Clone, Debug)]
    struct Snip {
        prefix: String,
        content: String,
    }

    let mut pre: BTreeMap<String, Snip> = BTreeMap::new();
    let mut deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for snip in snips {
        for name in &snip.attrs.names {
            let s = pre.entry(name.clone()).or_default();
            s.prefix += &snip.attrs.prefix;
            s.content += &snip.content;

            for dep in &snip.attrs.uses {
                deps.entry(name.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(dep.clone());
            }
        }
    }

    let mut res: BTreeMap<String, Snip> = BTreeMap::new();

    for (name, uses) in &deps {
        let mut used = HashSet::new();
        used.insert(name.clone());
        let mut stack = uses.iter().cloned().collect::<Vec<_>>();

        while let Some(dep) = stack.pop() {
            if !used.contains(&dep) {
                used.insert(dep.clone());
                if let Some(c) = &pre.get(&dep) {
                    // *res.entry(name.clone()).or_insert_with(String::new) += c.as_str();
                    let s = res.entry(name.clone()).or_default();
                    s.prefix += &c.prefix;
                    s.content += &c.content;

                    if let Some(ds) = deps.get(&dep) {
                        for d in ds {
                            if !used.contains(d) {
                                stack.push(d.clone());
                            }
                        }
                    }
                } else {
                    log::warn!("Dependency {} is missing", &dep);
                }
            }
        }
    }

    for (name, snip) in pre {
        // Dependency first
        let s = res.entry(name).or_default();
        s.prefix += snip.prefix.as_str();
        s.content += snip.content.as_str();
    }

    res.into_iter()
        .map(|(k, v)| (k, v.prefix + v.content.as_str()))
        .collect()
}

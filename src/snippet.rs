use std::collections::{BTreeMap, BTreeSet, HashSet};

pub struct SnippetAttributes {
    // A snippet with multiple names is allowed but using dependency is recommended.
    pub names: HashSet<String>,
    // Dependencies
    pub uses: HashSet<String>,
}

pub struct Snippet {
    pub attrs: SnippetAttributes,
    // Snippet content (Not formated)
    pub content: String,
}

pub fn process_snippets(snips: &[Snippet]) -> BTreeMap<String, String> {
    let mut pre: BTreeMap<String, String> = BTreeMap::new();
    let mut deps: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for snip in snips {
        for name in &snip.attrs.names {
            *pre.entry(name.clone()).or_insert_with(String::new) += &snip.content;

            for dep in &snip.attrs.uses {
                deps.entry(name.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(dep.clone());
            }
        }
    }

    let mut res: BTreeMap<String, String> = BTreeMap::new();

    for (name, uses) in &deps {
        let mut used = HashSet::new();
        used.insert(name.clone());
        let mut stack = uses.iter().cloned().collect::<Vec<_>>();

        while let Some(dep) = stack.pop() {
            if !used.contains(&dep) {
                used.insert(dep.clone());
                if let Some(c) = &pre.get(&dep) {
                    *res.entry(name.clone()).or_insert_with(String::new) += c.as_str();

                    if let Some(ds) = deps.get(&dep) {
                        for d in ds {
                            if !used.contains(d) {
                                stack.push(d.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    for (name, content) in pre {
        // Dependency first
        *res.entry(name).or_insert_with(String::new) += content.as_str();
    }

    res
}

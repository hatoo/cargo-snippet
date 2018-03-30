use quote::ToTokens;
use syn::{parse_file, Attribute, File, Item, Meta, NestedMeta};

macro_rules! get_attrs_impl {
    ($arg: expr, $($v: path), *) => {
        {
            match $arg {
                $(
                    &$v(ref x) => Some(&x.attrs),
                )*
                _ => None
            }
        }
    }
}

fn get_attrs(item: &Item) -> Option<&Vec<Attribute>> {
    // All Item variants except Item::Verbatim
    get_attrs_impl!(
        item,
        Item::ExternCrate,
        Item::Use,
        Item::Static,
        Item::Const,
        Item::Fn,
        Item::Mod,
        Item::ForeignMod,
        Item::Type,
        Item::Struct,
        Item::Enum,
        Item::Union,
        Item::Trait,
        Item::Impl,
        Item::Macro,
        Item::Macro2
    )
}

macro_rules! remove_snippet_attr_impl {
    ($arg: expr, $($v: path), *) => {
        {
            match $arg {
                $(
                    &mut $v(ref mut x) => {
                        x.attrs.retain(|attr| {
                            attr.interpret_meta().map(|m| m.name().to_string() != "snippet").unwrap_or(true)
                        });
                    },
                )*
                _ => ()
            }
        }
    }
}

fn remove_snippet_attr(item: &mut Item) {
    remove_snippet_attr_impl!(
        item,
        Item::ExternCrate,
        Item::Use,
        Item::Static,
        Item::Const,
        Item::Fn,
        Item::Mod,
        Item::ForeignMod,
        Item::Type,
        Item::Struct,
        Item::Enum,
        Item::Union,
        Item::Trait,
        Item::Impl,
        Item::Macro,
        Item::Macro2
    );

    match item {
        &mut Item::Mod(ref mut item_mod) => {
            if let Some(&mut (_, ref mut items)) = item_mod.content.as_mut() {
                items.iter_mut().for_each(|item| remove_snippet_attr(item));
            }
        }
        _ => (),
    }
}

fn unquote(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();

    if chars.len() >= 2 && chars.first() == Some(&'"') && chars.last() == Some(&'"') {
        chars[1..chars.len() - 1].iter().collect()
    } else {
        chars.iter().collect()
    }
}

fn get_snippet_name(attr: &Attribute) -> Option<String> {
    attr.interpret_meta().and_then(|metaitem| {
        if metaitem.name().to_string() != "snippet" {
            return None;
        }

        match metaitem {
            // #[snippet(name="..")]
            Meta::List(list) => list.nested
                .iter()
                .filter_map(|item| {
                    if let &NestedMeta::Meta(Meta::NameValue(ref nv)) = item {
                        if nv.ident.to_string() == "name" {
                            Some(unquote(&nv.lit.clone().into_tokens().to_string()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .next(),
            // #[snippet=".."]
            Meta::NameValue(nv) => Some(unquote(&nv.lit.into_tokens().to_string())),
            _ => None,
        }
    })
}

// Get snippet names and snippet code (not formatted)
fn get_snippet_from_item(mut item: Item) -> Option<(Vec<String>, String)> {
    let snip_names = get_attrs(&item).map(|attrs| {
        attrs
            .iter()
            .filter_map(|attr| get_snippet_name(attr))
            .collect()
    });

    snip_names.map(|names| {
        remove_snippet_attr(&mut item);
        (names, item.into_tokens().to_string())
    })
}

fn get_snippet_from_item_recursive(item: Item) -> Vec<(Vec<String>, String)> {
    let mut res = Vec::new();

    if let Some(pair) = get_snippet_from_item(item.clone()) {
        res.push(pair);
    }

    if let Item::Mod(mod_item) = item {
        res.extend(
            mod_item
                .content
                .into_iter()
                .flat_map(|(_, items)| items.into_iter().flat_map(get_snippet_from_item_recursive)),
        );
    }

    res
}

fn get_snippet_from_file(file: File) -> Vec<(String, String)> {
    let mut res = Vec::new();

    // whole code is snippet
    let snip_names = file.attrs
        .iter()
        .filter_map(|attr| get_snippet_name(attr))
        .collect::<Vec<_>>();

    for name in snip_names {
        let mut file = file.clone();
        file.attrs.retain(|attr| {
            attr.interpret_meta()
                .map(|m| m.name().to_string() != "snippet")
                .unwrap_or(true)
        });
        file.items.iter_mut().for_each(|item| {
            remove_snippet_attr(item);
        });
        res.push((name, file.into_tokens().to_string()));
    }

    res.extend(
        file.items
            .into_iter()
            .flat_map(|item| get_snippet_from_item_recursive(item))
            .flat_map(|(names, content)| {
                names.into_iter().map(move |name| (name, content.clone()))
            }),
    );

    res
}

pub fn parse_snippet(src: &str) -> Vec<(String, String)> {
    parse_file(src)
        .ok()
        .map(|file| get_snippet_from_file(file))
        .unwrap_or(Vec::new())
}

#[cfg(test)]
mod test {
    use super::parse_snippet;
    use std::collections::BTreeMap;

    fn snippets(src: &str) -> BTreeMap<String, String> {
        let mut res = BTreeMap::new();
        for (name, content) in parse_snippet(src) {
            *res.entry(name).or_insert(String::new()) += &content;
        }
        res
    }

    #[test]
    fn test_parse_simple_case() {
        let src = r#"
            #[snippet="test"]
            fn test() {}
        "#;

        let snip = snippets(&src);

        assert_eq!(snip.get("test"), Some(&quote!(fn test() {}).to_string()));
    }

    #[test]
    fn test_multiple_annotaton() {
        {
            let src = r#"
                #[snippet="test1"]
                #[snippet="test2"]
                fn test() {}
            "#;

            let snip = snippets(&src);

            assert_eq!(snip.get("test1"), Some(&quote!(fn test() {}).to_string()));
            assert_eq!(snip.get("test2"), Some(&quote!(fn test() {}).to_string()));
        }

        {
            let src = r#"
                #![snippet="test1"]
                #![snippet="test2"]

                fn test() {}
            "#;

            let snip = snippets(&src);

            assert_eq!(snip.get("test1"), Some(&quote!(fn test() {}).to_string()));
            assert_eq!(snip.get("test2"), Some(&quote!(fn test() {}).to_string()));
        }
    }

    #[test]
    fn test_deep() {
        let src = r#"
            #[snippet = "bar"]
            fn bar() {}

            #[snippet = "foo"]
            mod foo {
                #[snippet = "hoge"]
                fn hoge() {}
            }
        "#;

        let snip = snippets(&src);

        assert_eq!(snip.get("bar"), Some(&quote!(fn bar() {}).to_string()));
        assert_eq!(
            snip.get("foo"),
            // #[snippet = "hoge"] should be removed.
            Some(&quote!(mod foo {
                fn hoge() {}
            }).to_string())
        );
        assert_eq!(snip.get("hoge"), Some(&quote!(fn hoge() {}).to_string()));
    }
}

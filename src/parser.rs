use quote::ToTokens;
use syn;
use syn::{parse_file, Attribute, File, Item, Meta, NestedMeta};

use crate::snippet::{Snippet, SnippetAttributes};
use std::collections::HashSet;

macro_rules! get_attrs_impl {
    ($arg: expr, $($v: path), *) => {
        {
            match $arg {
                $(
                    $v(ref x) => Some(&x.attrs),
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
                    $v(ref mut x) => {
                        x.attrs.retain(|attr| {
                            attr.parse_meta().map(|m| m.path().to_token_stream().to_string() != "snippet").unwrap_or(true)
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

    if let Item::Mod(ref mut item_mod) = item {
        if let Some(&mut (_, ref mut items)) = item_mod.content.as_mut() {
            items.iter_mut().for_each(|item| remove_snippet_attr(item));
        }
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

macro_rules! get_default_snippet_name_impl {
    ($arg:expr, $($v: path), *) => {
        match $arg {
            $(
                $v(ref x) => {
                    Some(x.ident.to_string())
                },
            )*
            Item::Fn(ref x) => {
                Some(x.sig.ident.to_string())
            }
            _ => None
        }
    };
}

fn get_default_snippet_name(item: &Item) -> Option<String> {
    get_default_snippet_name_impl!(
        item,
        Item::Static,
        Item::Const,
        Item::Mod,
        Item::Struct,
        Item::Enum,
        Item::Union,
        Item::Trait
    )
}

fn get_snippet_name(attr: &Attribute) -> Option<String> {
    attr.parse_meta().ok().and_then(|metaitem| {
        if metaitem.path().to_token_stream().to_string() != "snippet" {
            return None;
        }

        match metaitem {
            // #[snippet(name="..")]
            Meta::List(list) => list
                .nested
                .iter()
                .filter_map(|item| match item {
                    NestedMeta::Meta(Meta::NameValue(ref nv)) => {
                        if nv.path.to_token_stream().to_string() == "name" {
                            Some(unquote(&nv.lit.clone().into_token_stream().to_string()))
                        } else {
                            None
                        }
                    }
                    NestedMeta::Lit(lit) => {
                        Some(unquote(lit.to_token_stream().to_string().as_str()))
                    }
                    _ => None,
                })
                .next(),
            // #[snippet=".."]
            Meta::NameValue(nv) => Some(unquote(&nv.lit.into_token_stream().to_string())),
            _ => None,
        }
    })
}

fn get_snippet_uses(attr: &Attribute) -> Option<Vec<String>> {
    attr.parse_meta().ok().and_then(|metaitem| {
        if metaitem.path().to_token_stream().to_string() != "snippet" {
            return None;
        }

        match metaitem {
            // #[snippet(include="..")]
            Meta::List(list) => list
                .nested
                .iter()
                .filter_map(|item| {
                    if let NestedMeta::Meta(Meta::NameValue(ref nv)) = item {
                        // It can't use "use" keyword here xD.
                        // It is reserved.
                        if nv.path.to_token_stream().to_string() == "include" {
                            let uses = unquote(&nv.lit.clone().into_token_stream().to_string());
                            Some(
                                uses.split(',')
                                    .map(|s| s.trim())
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .next(),
            _ => None,
        }
    })
}

fn parse_attrs(
    attrs: &[Attribute],
    default_snippet_name: Option<String>,
) -> Option<SnippetAttributes> {
    if !attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .any(|m| m.path().to_token_stream().to_string() == "snippet")
    {
        return None;
    }

    let mut names = attrs
        .iter()
        .filter_map(get_snippet_name)
        .collect::<HashSet<_>>();

    let attr_snippet_without_value = attrs.iter().filter_map(|a| a.parse_meta().ok()).any(|m| {
        if m.path().to_token_stream().to_string() != "snippet" {
            return false;
        }

        match m {
            syn::Meta::Path(_) => true,
            _ => false,
        }
    });

    if attr_snippet_without_value {
        if let Some(ref default) = default_snippet_name {
            names.insert(default.clone());
        }
    }

    if names.is_empty() {
        if let Some(default) = default_snippet_name {
            names.insert(default);
        } else {
            return None;
        }
    }

    let uses = attrs
        .iter()
        .filter_map(get_snippet_uses)
        .flat_map(|v| v.into_iter())
        .collect::<HashSet<_>>();

    Some(SnippetAttributes { names, uses })
}

// Get snippet names and snippet code (not formatted)
fn get_snippet_from_item(mut item: Item) -> Option<Snippet> {
    let default_name = get_default_snippet_name(&item);
    let snip_attrs = get_attrs(&item).and_then(|attrs| parse_attrs(attrs.as_slice(), default_name));

    snip_attrs.map(|attrs| {
        remove_snippet_attr(&mut item);
        Snippet {
            attrs,
            content: item.into_token_stream().to_string(),
        }
    })
}

fn get_snippet_from_item_recursive(item: Item) -> Vec<Snippet> {
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

fn get_snippet_from_file(file: File) -> Vec<Snippet> {
    let mut res = Vec::new();

    // whole code is snippet
    if let Some(attrs) = parse_attrs(&file.attrs, None) {
        let mut file = file.clone();
        file.attrs.retain(|attr| {
            attr.parse_meta()
                .map(|m| m.path().to_token_stream().to_string() != "snippet")
                .unwrap_or(true)
        });
        file.items.iter_mut().for_each(|item| {
            remove_snippet_attr(item);
        });
        res.push(Snippet {
            attrs,
            content: file.into_token_stream().to_string(),
        })
    }

    res.extend(
        file.items
            .into_iter()
            .flat_map(get_snippet_from_item_recursive),
    );

    res
}

pub fn parse_snippet(src: &str) -> Result<Vec<Snippet>, syn::parse::Error> {
    parse_file(src).map(get_snippet_from_file)
}

#[cfg(test)]
mod test {
    use super::parse_snippet;
    use crate::snippet::process_snippets;
    use crate::writer::format_src;
    use quote::quote;
    use std::collections::BTreeMap;

    fn snippets(src: &str) -> BTreeMap<String, String> {
        let snips = parse_snippet(src).unwrap();
        process_snippets(&snips)
    }

    #[test]
    fn test_no_snippet() {
        let src = r#"
            fn test() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(snip.get("test"), None);
    }

    #[test]
    fn test_parse_simple_case() {
        let src = r#"
            #[snippet("test")]
            fn test() {}
        "#;

        let snip = snippets(&src);

        assert_eq!(
            snip.get("test"),
            Some(
                &quote!(
                    fn test() {}
                )
                .to_string()
            )
        );
    }

    #[test]
    fn test_multiple_annotaton() {
        {
            let src = r#"
                #[snippet("test1")]
                #[snippet("test2")]
                fn test() {}
            "#;

            let snip = snippets(&src);

            dbg!(&snip);

            assert_eq!(
                snip.get("test1"),
                Some(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("test2"),
                Some(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
        }

        {
            let src = r#"
                #![snippet("test1")]
                #![snippet("test2")]

                fn test() {}
            "#;

            let snip = snippets(&src);

            assert_eq!(
                snip.get("test1"),
                Some(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("test2"),
                Some(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
        }

        {
            let src = r#"
                #[snippet]
                #[snippet("bar2")]
                fn bar() {}
            "#;

            let snip = snippets(&src);
            assert_eq!(
                snip.get("bar"),
                Some(
                    &quote!(
                        fn bar() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("bar2"),
                Some(
                    &quote!(
                        fn bar() {}
                    )
                    .to_string()
                )
            );
        }
    }

    #[test]
    fn test_deep() {
        let src = r#"
            #[snippet("bar")]
            fn bar() {}

            #[snippet("foo")]
            mod foo {
                #[snippet("hoge")]
                fn hoge() {}
            }
        "#;

        let snip = snippets(&src);

        assert_eq!(
            snip.get("bar"),
            Some(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("foo"),
            // #[snippet = "hoge"] should be removed.
            Some(
                &quote!(
                    mod foo {
                        fn hoge() {}
                    }
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("hoge"),
            Some(
                &quote!(
                    fn hoge() {}
                )
                .to_string()
            )
        );
    }

    #[test]
    fn test_default_snippet_name() {
        let src = r#"
            #[snippet]
            fn bar() {}

            #[snippet]
            struct Baz();
        "#;

        let snip = snippets(&src);
        assert_eq!(
            snip.get("bar"),
            Some(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("Baz"),
            Some(
                &quote!(
                    struct Baz();
                )
                .to_string()
            )
        );
    }

    #[test]
    fn test_snippet_dependency() {
        let src = r#"
            #[snippet("bar")]
            fn bar() {}

            #[snippet(name = "baz", include = "bar")]
            fn baz() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            snip.get("bar"),
            Some(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        assert_eq!(
            format_src(snip["baz"].as_str()).unwrap(),
            format_src("fn bar() {} fn baz() {}").unwrap()
        );

        let src = r#"
            #[snippet]
            fn foo() {}

            #[snippet]
            fn bar() {}

            #[snippet(name = "baz", include = "foo, bar")]
            fn baz() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            snip.get("bar"),
            Some(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        // Original order of "uses" are not saved.
        assert_eq!(
            format_src(snip["baz"].as_str()).unwrap(),
            format_src("fn foo() {} fn bar() {} fn baz() {}").unwrap()
        );
    }

    #[test]
    fn test_recursive_dependency() {
        let src = r#"
            #[snippet(include = "baz")]
            fn bar() {}

            #[snippet(include = "bar")]
            fn baz() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("fn baz() {} fn bar() {}").unwrap()
        );
        assert_eq!(
            format_src(snip["baz"].as_str()).unwrap(),
            format_src("fn bar() {} fn baz() {}").unwrap()
        );
    }

    #[test]
    fn test_missing_dependency() {
        let src = r#"
            #[snippet(include = "foo")]
            fn bar() {}

            #[snippet(include = "foo")]
            fn baz() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("fn bar() {}").unwrap()
        );
        assert_eq!(
            format_src(snip["baz"].as_str()).unwrap(),
            format_src("fn baz() {}").unwrap()
        );
    }
}

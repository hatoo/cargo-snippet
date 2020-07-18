use lazy_static::lazy_static;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use regex::{Captures, Regex};
use syn::{parse_file, Attribute, File, Item, Meta, MetaList, NestedMeta};

use crate::snippet::{Snippet, SnippetAttributes};
use std::collections::HashSet;
use std::{char, u32};

fn is_snippet_path(path: &str) -> bool {
    match path {
        "snippet" | "cargo_snippet :: snippet" => true,
        _ => false,
    }
}

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
                            attr.parse_meta().map(|m| !is_snippet_path(m.path().to_token_stream().to_string().as_str())).unwrap_or(true)
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
        if !is_snippet_path(metaitem.path().to_token_stream().to_string().as_str()) {
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
        if !is_snippet_path(metaitem.path().to_token_stream().to_string().as_str()) {
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

fn get_simple_attr(attr: &Attribute, key: &str) -> Vec<String> {
    attr.parse_meta()
        .ok()
        .and_then(|metaitem| {
            if !is_snippet_path(metaitem.path().to_token_stream().to_string().as_str()) {
                return None;
            }

            match metaitem {
                // #[snippet(`key`="..")]
                Meta::List(list) => list
                    .nested
                    .iter()
                    .filter_map(|item| {
                        if let NestedMeta::Meta(Meta::NameValue(ref nv)) = item {
                            if nv.path.to_token_stream().to_string() == key {
                                let value = if let syn::Lit::Str(s) = &nv.lit.clone() {
                                    s.value()
                                } else {
                                    panic!("attribute must be string");
                                };
                                Some(value)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .into(),
                _ => None,
            }
        })
        .unwrap_or(Vec::new())
}

fn parse_attrs(
    attrs: &[Attribute],
    default_snippet_name: Option<String>,
) -> Option<SnippetAttributes> {
    if !attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .any(|m| is_snippet_path(m.path().to_token_stream().to_string().as_str()))
    {
        return None;
    }

    let mut names = attrs
        .iter()
        .filter_map(get_snippet_name)
        .collect::<HashSet<_>>();

    let attr_snippet_without_value = attrs.iter().filter_map(|a| a.parse_meta().ok()).any(|m| {
        if !is_snippet_path(m.path().to_token_stream().to_string().as_str()) {
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

    let prefix = attrs
        .iter()
        .map(|attr| get_simple_attr(attr, "prefix").into_iter())
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");

    let doc_hidden = attrs.iter().filter_map(|a| a.parse_meta().ok()).any(|m| {
        let is_snippet = is_snippet_path(m.path().to_token_stream().to_string().as_str());
        let doc_hidden = match m {
            Meta::List(MetaList { ref nested, .. }) => nested.iter().any(|n| match n {
                NestedMeta::Meta(Meta::Path(ref p)) => {
                    p.to_token_stream().to_string() == "doc_hidden"
                }
                _ => false,
            }),
            _ => false,
        };
        is_snippet && doc_hidden
    });

    Some(SnippetAttributes {
        names,
        uses,
        prefix,
        doc_hidden,
    })
}

fn next_token_is_doc(token: &TokenTree) -> bool {
    match token {
        TokenTree::Group(ref g) => g.to_string().starts_with("[doc = "),
        _ => false,
    }
}

fn unescape(s: impl Into<String>) -> String {
    lazy_static! {
        static ref ESCAPED_UNICODE: Regex = Regex::new(r"\\u\{([0-9a-fA-F]{1,6})\}").unwrap();
    }
    let s = s.into();
    let unicode_unescaped: Vec<char> = ESCAPED_UNICODE
        .replace_all(&s, |caps: &Captures| {
            caps.get(1)
                .and_then(|cap| u32::from_str_radix(cap.as_str(), 16).ok())
                .and_then(|u| char::from_u32(u))
                .map(|ch| ch.to_string())
                .unwrap_or(caps[0].to_string())
        })
        .chars()
        .collect();

    let mut ret = String::with_capacity(s.len());
    let mut iter = unicode_unescaped.iter().peekable();
    while let Some(&ch) = iter.next() {
        if ch == '\\' {
            match iter.peek() {
                Some(&next_ch) if *next_ch == '\\' => {
                    ret.push('\\');
                    iter.next();
                }
                Some(&next_ch) if *next_ch == '"' => {
                    ret.push('"');
                    iter.next();
                }
                Some(&next_ch) if *next_ch == 't' => {
                    ret.push('\t');
                    iter.next();
                }
                Some(&next_ch) if *next_ch == 'n' => {
                    ret.push('\n');
                    iter.next();
                }
                Some(&next_ch) if *next_ch == 'r' => {
                    ret.push('\r');
                    iter.next();
                }
                _ => unreachable!(),
            }
        } else {
            ret.push(ch);
        }
    }
    ret
}

fn format_doc_comment(doc_tt: TokenTree, is_inner: bool, doc_hidden: bool) -> Option<String> {
    lazy_static! {
        static ref DOC_RE: Regex = Regex::new(r#"^\[doc = "(?s)(.*)"\]$"#).unwrap();
    }
    if doc_hidden {
        return None;
    }

    let doc = unescape(doc_tt.to_string());
    DOC_RE
        .captures(doc.as_str())
        .and_then(|caps| caps.get(1))
        .map(|c| {
            c.as_str().lines().fold(String::new(), |mut acc, line| {
                let s = if is_inner {
                    format!("//!{}\n", line)
                } else {
                    format!("///{}\n", line)
                };
                acc.push_str(&s);
                acc
            })
        })
}

fn stringify_tokens(tokens: TokenStream, doc_hidden: bool) -> String {
    let mut res = String::new();
    let mut iter = tokens.into_iter().peekable();
    while let Some(tok) = iter.next() {
        match tok {
            TokenTree::Punct(ref punct) => {
                if punct.as_char() == '!' && iter.peek().map(next_token_is_doc).unwrap_or(false) {
                    // inner doc comment here.
                    // `res` already has a `#` character at the last, which is unnecessary, so remove it by calling pop.
                    if res.chars().last() == Some(' ') {
                        res.pop();
                    }
                    assert_eq!(res.pop(), Some('#'));
                    if let Some(doc) =
                        format_doc_comment(iter.next().unwrap(), true, doc_hidden).as_deref()
                    {
                        res.push_str(doc);
                    }
                } else if punct.as_char() == '#'
                    && iter.peek().map(next_token_is_doc).unwrap_or(false)
                {
                    // outer doc comment here.
                    if let Some(doc) =
                        format_doc_comment(iter.next().unwrap(), false, doc_hidden).as_deref()
                    {
                        res.push_str(doc);
                    }
                } else {
                    res.push_str(tok.to_string().as_str());
                    if punct.spacing() == proc_macro2::Spacing::Alone {
                        res.push(' ');
                    }
                }
            }
            TokenTree::Group(ref g) => {
                match g.delimiter() {
                    Delimiter::Parenthesis => res.push('('),
                    Delimiter::Brace => res.push('{'),
                    Delimiter::Bracket => res.push('['),
                    Delimiter::None => (),
                }
                res.push_str(stringify_tokens(g.stream(), doc_hidden).as_str());
                match g.delimiter() {
                    Delimiter::Parenthesis => res.push(')'),
                    Delimiter::Brace => res.push('}'),
                    Delimiter::Bracket => res.push(']'),
                    Delimiter::None => (),
                }
                res.push(' ');
            }
            _ => {
                res.push_str(tok.to_string().as_str());
                res.push(' ');
            }
        }
    }
    res
}

// Get snippet names and snippet code (not formatted)
fn get_snippet_from_item(mut item: Item) -> Option<Snippet> {
    let default_name = get_default_snippet_name(&item);
    let snip_attrs = get_attrs(&item).and_then(|attrs| parse_attrs(attrs.as_slice(), default_name));

    snip_attrs.map(|attrs| {
        remove_snippet_attr(&mut item);
        let doc_hidden = attrs.doc_hidden;
        Snippet {
            attrs,
            content: stringify_tokens(item.into_token_stream(), doc_hidden),
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
                .map(|m| !is_snippet_path(m.path().to_token_stream().to_string().as_str()))
                .unwrap_or(true)
        });
        file.items.iter_mut().for_each(|item| {
            remove_snippet_attr(item);
        });
        let doc_hidden = attrs.doc_hidden;
        res.push(Snippet {
            attrs,
            content: stringify_tokens(file.into_token_stream(), doc_hidden),
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
    use super::{parse_snippet, unescape};
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
            snip.get("test").and_then(|s| format_src(s)),
            format_src(
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

            assert_eq!(
                snip.get("test1").and_then(|s| format_src(s)),
                format_src(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("test2").and_then(|s| format_src(s)),
                format_src(
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
                snip.get("test1").and_then(|s| format_src(s)),
                format_src(
                    &quote!(
                        fn test() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("test2").and_then(|s| format_src(s)),
                format_src(
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
                snip.get("bar").and_then(|s| format_src(s)),
                format_src(
                    &quote!(
                        fn bar() {}
                    )
                    .to_string()
                )
            );
            assert_eq!(
                snip.get("bar2").and_then(|s| format_src(s)),
                format_src(
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
            snip.get("bar").and_then(|s| format_src(s)),
            format_src(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("foo").and_then(|s| format_src(s)),
            // #[snippet("hoge")] should be removed.
            format_src(
                &quote!(
                    mod foo {
                        fn hoge() {}
                    }
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("hoge").and_then(|s| format_src(s)),
            format_src(
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
            snip.get("bar").and_then(|s| format_src(s)),
            format_src(
                &quote!(
                    fn bar() {}
                )
                .to_string()
            )
        );
        assert_eq!(
            snip.get("Baz").and_then(|s| format_src(s)),
            format_src(
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
            snip.get("bar").and_then(|s| format_src(s)),
            format_src(
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
            snip.get("bar").and_then(|s| format_src(s)),
            format_src(
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

    #[test]
    fn test_attribute_full_path() {
        let src = r#"
            #[cargo_snippet::snippet]
            fn bar() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("fn bar() {}").unwrap()
        );
    }

    #[test]
    fn test_attribute_prefix() {
        let src = r#"
            #[snippet(prefix = "use std::io;")]
            fn bar() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("use std::io;\nfn bar() {}").unwrap()
        );

        let src = r#"
            #[snippet(prefix="use std::io::{self,Read};\nuse std::str::FromStr;")]
            fn bar() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("use std::io::{self,Read};\nuse std::str::FromStr;\nfn bar() {}").unwrap()
        );

        let src = r#"
            #[snippet(prefix=r"use std::io::{self,Read};
use std::str::FromStr;")]
            fn bar() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("use std::io::{self,Read};\nuse std::str::FromStr;\nfn bar() {}").unwrap()
        );
    }

    #[test]
    fn test_attribute_prefix_include() {
        let src = r#"
            #[snippet(prefix = "use std::sync;")]
            fn foo() {}
            #[snippet(prefix = "use std::io;", include = "foo")]
            fn bar() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src(
                &quote!(
                    use std::sync;
                    use std::io;
                    fn foo() {}
                    fn bar() {}
                )
                .to_string()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_outer_line_doc() {
        let src = r#"
            /// This is outer doc comment. (exactly three slashes)
            // This is *NOT* doc comment.
            //// This is also *NOT* doc comment.
            #[snippet]
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("/// This is outer doc comment. (exactly three slashes)\nfn foo() {}")
                .unwrap(),
        );
    }

    #[test]
    fn test_outer_block_doc() {
        let src = r#"
/** This is outer doc comment.
doc comment1
* doc comment2
 doc comment finishes here! */
/*
NOT doc comment
*/
/*** NOT doc comment */
#[snippet]
fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src(
                r#"
/// This is outer doc comment.
///doc comment1
///* doc comment2
/// doc comment finishes here!
fn foo() {}
"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_inner_line_doc() {
        let src = r#"
            #[snippet]
            fn foo() {
                //! This is inner doc comment.
            }
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {\n//! This is inner doc comment.\n}").unwrap(),
        );
    }

    #[test]
    fn test_inner_block_doc() {
        let src = r#"
#[snippet]
fn foo() {
/*! This is inner doc comment.
doc comment1
* doc comment2
 doc comment finishes here! */
/*
NOT doc comment
*/
/*** NOT doc comment */
}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src(
                r#"
fn foo() {
//! This is inner doc comment.
//!doc comment1
//!* doc comment2
//! doc comment finishes here!
}
"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_outer_line_doc_in_file() {
        let src = r#"
            #![snippet("file")]
            /// This is outer doc comment.
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["file"].as_str()).unwrap(),
            format_src("/// This is outer doc comment.\nfn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_outer_line_doc_in_file_escaped_chars() {
        let src = r#"
             #![snippet("file")]
             /// ///\\\ 'This \t is \r outer " doc \n comment.
             fn foo() {}
         "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["file"].as_str()).unwrap(),
            format_src(
                r#"/// ///\\\ 'This \t is \r outer " doc \n comment.
             fn foo() {}"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_inner_line_doc_in_file() {
        let src = r#"
            #![snippet("file")]
            //! This is inner doc comment.
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["file"].as_str()).unwrap(),
            format_src("//! This is inner doc comment.\nfn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_inner_line_doc_in_file_backslash() {
        let src = r#"
            #![snippet("file")]
            //! ///\\\ This is outer doc comment.
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["file"].as_str()).unwrap(),
            format_src(
                r#"//! ///\\\ This is outer doc comment.
            fn foo() {}"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_inner_line_doc_in_file_tab() {
        let src = r#"
            #![snippet("file")]
            //! /// 	<- tab character
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["file"].as_str()).unwrap(),
            format_src(
                r#"//! /// 	<- tab character
            fn foo() {}"#
            )
            .unwrap(),
        );
    }

    #[test]
    fn test_unicode_unescape() {
        // cf. https://ja.wikipedia.org/wiki/%E3%82%B9%E3%83%9A%E3%83%BC%E3%82%B9
        assert_eq!(unescape("foo\\u{2002}bar"), "foo bar"); // EN SPACE
        assert_eq!(unescape("foo\\u{2003}bar"), "foo bar"); // EM SPACE
        assert_eq!(unescape("foo\\u{2004}bar"), "foo bar"); // THREE-PER-EM SPACE
        assert_eq!(unescape("foo\\u{2005}bar"), "foo bar"); // FOUR-PER-EM SPACE
        assert_eq!(unescape("foo\\u{2006}bar"), "foo bar"); // SIX-PER-EM SPACE
        assert_eq!(unescape("foo\\u{2007}bar"), "foo bar"); // FIGURE SPACE
        assert_eq!(unescape("foo\\u{2008}bar"), "foo bar"); // PUNCTUATION SPACE
        assert_eq!(unescape("foo\\u{2009}bar"), "foo bar"); // THIN SPACE
        assert_eq!(unescape("foo\\u{200A}bar"), "foo bar"); // HAIR SPACE
        assert_eq!(unescape("foo\\u{200B}bar"), "foo\u{200B}bar"); // ZERO WIDTH SPACE
        assert_eq!(unescape("foo\\u{3000}bar"), "foo　bar"); // IDEOGRAPHIC SPACE
    }

    #[test]
    fn test_full_width_space_in_outer_line_doc() {
        let src = r#"
            #[snippet]
            /// [　] <- full width space
            fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("/// [　] <- full width space\nfn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_full_width_space_in_outer_block_doc() {
        let src = r#"
#[snippet]
/** 
[　] <- full width space
*/
fn foo() {}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("///\n///[　] <- full width space\nfn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_full_width_space_in_inner_line_doc() {
        let src = r#"
            #[snippet]
            fn foo() {
                //! [　] <- full width space
            }
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {\n//! [　] <- full width space\n}").unwrap(),
        );
    }

    #[test]
    fn test_full_width_space_in_inner_block_doc() {
        let src = r#"
#[snippet]
fn foo() {
/*!
[　] <- full width space
*/
}
        "#;

        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {\n//!\n//![　] <- full width space\n}").unwrap(),
        );
    }

    #[test]
    fn test_divide_deref() {
        let src = r#"
#[snippet]
fn foo(a: &i32, b: &i32) -> i32 {
    *a / *b
}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo(a: &i32, b: &i32) -> i32 { *a / *b }").unwrap(),
        );
    }

    #[test]
    fn test_doc_hidden_outer_line() {
        let src = r#"
/// comment
#[snippet(doc_hidden)]
fn foo() {}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_doc_hidden_inner_line() {
        let src = r#"
#[snippet(doc_hidden)]
fn foo() {
    //! comment
}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_doc_hidden_outer_block() {
        let src = r#"
/** comment */
#[snippet(doc_hidden)]
fn foo() {}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_doc_hidden_inner_block() {
        let src = r#"
#[snippet(doc_hidden)]
fn foo() {
    /*! comment */
}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["foo"].as_str()).unwrap(),
            format_src("fn foo() {}").unwrap(),
        );
    }

    #[test]
    fn test_doc_hidden_outer_line_with_other_metas() {
        let src = r#"
/// comment
#[snippet(name = "bar", doc_hidden, prefix = "use std::collections::HashMap;")]
fn foo() {}
        "#;
        let snip = snippets(&src);
        assert_eq!(
            format_src(snip["bar"].as_str()).unwrap(),
            format_src("use std::collections::HashMap;\nfn foo() {}").unwrap(),
        );
    }
}

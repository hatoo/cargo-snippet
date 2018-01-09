#![crate_type = "dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::feature_gate::AttributeType;

#[plugin_registrar]
/// Register "snippet" attribute to the compiler ignore it.
///
/// This function is called automatically.
///
/// See [here](https://doc.rust-lang.org/nightly/unstable-book/language-features/plugin.html).
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_attribute("snippet".to_owned(), AttributeType::Whitelisted);
}

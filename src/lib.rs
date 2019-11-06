#![crate_type = "dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate rustc_driver;
extern crate rustc_plugin;
extern crate syntax;
extern crate syntax_pos;

use rustc_plugin::Registry;
use syntax::feature_gate::AttributeType;
use syntax_pos::symbol::Symbol;

#[plugin_registrar]
/// Register "snippet" attribute to the compiler ignore it.
///
/// This function is called automatically.
///
/// See [here](https://doc.rust-lang.org/nightly/unstable-book/language-features/plugin.html).
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_attribute(Symbol::intern("snippet"), AttributeType::Whitelisted);
}

#![crate_type = "dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::Registry;
use syntax::feature_gate::AttributeType;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_attribute("snippet".to_owned(), AttributeType::Whitelisted);
}

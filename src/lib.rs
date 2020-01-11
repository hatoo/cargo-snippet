extern crate proc_macro;

use crate::proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn snippet(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#![recursion_limit="256"]

extern crate proc_macro;
extern crate relm_derive_common;
extern crate syn;

use proc_macro::TokenStream;

use relm_derive_common::{impl_msg, impl_simple_msg};
use syn::{Ident, parse_macro_input};

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_simple_msg(&ast, Ident::new("relm_state"));
    gen.parse().unwrap()
}

#[proc_macro_derive(Msg)]
pub fn msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_msg(&ast, Ident::new("relm_state"));
    gen.parse().unwrap()
}

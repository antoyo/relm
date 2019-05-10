#![recursion_limit="256"]

extern crate proc_macro;
extern crate relm_derive_common;
extern crate syn;

use proc_macro::TokenStream;

use relm_derive_common::{impl_msg, impl_simple_msg};
use syn::{Ident, Item, parse};
use syn::spanned::Spanned;

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let ast: Item = parse(input).expect("simple_msg > parse failed");
    let gen = impl_simple_msg(&ast, Ident::new("relm::state", ast.span()));
    gen.into()
}

#[proc_macro_derive(Msg)]
pub fn msg(input: TokenStream) -> TokenStream {
    let ast: Item = parse(input).expect("msg > parse failed");
    let gen = impl_msg(&ast, Ident::new("relm::state", ast.span()));
    gen.into()
}

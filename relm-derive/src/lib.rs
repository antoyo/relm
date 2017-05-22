/*
 * TODO: does an attribute #[msg] would simplify the implementation instead of #[derive(Msg)]?
 */

#![recursion_limit="256"]

extern crate proc_macro;
extern crate quote;
extern crate relm_derive_common;
extern crate relm_gen_widget;
extern crate syn;

use proc_macro::TokenStream;

use quote::Tokens;
use relm_gen_widget::gen_widget;
use relm_derive_common::{impl_msg, impl_simple_msg};
use syn::{
    Ident,
    Item,
    VariantData,
    parse_item,
    parse_macro_input,
};
use syn::ItemKind::Struct;
use syn::TokenTree::Delimited;
use syn::Ty::Mac;

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_simple_msg(&ast, Ident::new("relm"));
    gen.parse().unwrap()
}

#[proc_macro_derive(Msg)]
pub fn msg(input: TokenStream) -> TokenStream {
    let string = input.to_string();
    let ast = parse_macro_input(&string).unwrap();
    let gen = impl_msg(&ast, Ident::new("relm"));
    gen.parse().unwrap()
}

#[proc_macro_derive(Widget)]
pub fn widget(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = parse_item(&source).unwrap();
    let expanded = impl_widget(&ast);
    expanded.parse().unwrap()
}

fn impl_widget(ast: &Item) -> Tokens {
    if let Struct(VariantData::Struct(ref fields), _) = ast.node {
        for field in fields {
            if field.ident == Some(Ident::new("widget")) {
                if let Mac(ref mac) = field.ty {
                    if let Delimited(syn::Delimited { ref tts, .. }) = mac.tts[0] {
                        let mut tokens = Tokens::new();
                        tokens.append_all(tts);
                        return gen_widget(tokens);
                    }
                }
            }
        }
    }

    panic!("Expecting `widget` field.");
}

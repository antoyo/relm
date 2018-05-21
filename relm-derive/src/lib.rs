/*
 * TODO: does an attribute #[msg] would simplify the implementation instead of #[derive(Msg)]?
 */

#![recursion_limit="256"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate relm_derive_common;
extern crate relm_gen_widget;
extern crate syn;

use proc_macro::TokenStream;
use relm_gen_widget::gen_widget;
use relm_derive_common::{impl_msg, impl_simple_msg};
use syn::{
    Ident,
    Item,
    ItemStruct,
    parse,
};
use syn::Type::Macro;
use syn::spanned::Spanned;

#[proc_macro_derive(SimpleMsg)]
pub fn simple_msg(input: TokenStream) -> TokenStream {
    let ast: Item = parse(input).unwrap();
    let gen = impl_simple_msg(&ast, Ident::new("relm", ast.span()));
    gen.into()
}

#[proc_macro_derive(Msg)]
pub fn msg(input: TokenStream) -> TokenStream {
    let ast: Item = parse(input).unwrap();
    let gen = impl_msg(&ast, Ident::new("relm", ast.span()));
    gen.into()
}

#[proc_macro_derive(Widget)]
pub fn widget(input: TokenStream) -> TokenStream {
    let ast: Item = parse(input).unwrap();
    let expanded = impl_widget(&ast);
    expanded.into()
}

fn impl_widget(ast: &Item) -> TokenStream {
    if let Item::Struct(ItemStruct { ref fields, ..}) = *ast {
        for field in fields {
            if let Some(ref ident) = field.ident {
                if ident == "widget" {
                    if let Macro(ref mac) = field.ty {
                        let tts = &mac.mac.tts;
                        let tokens = quote! {
                            #tts
                        };
                        return gen_widget(tokens.into()).into();
                    }
                }
            }
        }
    }

    panic!("Expecting `widget` field.");
}

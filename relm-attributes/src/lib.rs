/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod gen;
mod parser;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};
use syn::{Delimited, Ident, ImplItem, TokenTree, parse_item};
use syn::ItemKind::Impl;
use syn::ImplItemKind::{Const, Macro, Method, Type};
use syn::Ty::Path;

use gen::gen;
use parser::parse;

#[proc_macro_attribute]
pub fn widget(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let mut ast = parse_item(&source).unwrap();
    if let Impl(unsafety, polarity, generics, path, typ, items) = ast.node {
        let name =
            if let Path(_, ref path) = *typ {
                // TODO: [0] might not be enough.
                path.segments[0].ident.clone()
            }
            else {
                panic!("Expected Path")
            };
        let mut new_items = vec![];
        let mut container_method = None;
        let mut root_widget = None;
        for item in items {
            let i = item.clone();
            let new_item =
                match item.node {
                    Const(_, _) => panic!("Unexpected const item"),
                    Macro(mac) => {
                        let segments = mac.path.segments;
                        if segments.len() == 1 && segments[0].ident.to_string() == "view" {
                            impl_view(&name, mac.tts, &mut root_widget)
                        }
                        else {
                            panic!("Unexpected macro item")
                        }
                    },
                    Method(_, _) => {
                        match item.ident.to_string().as_ref() {
                            "container" => {
                                container_method = Some(i);
                                continue;
                            },
                            "model" => i,
                            "subscriptions" => i,
                            "update" => i, // TODO: automatically add the widget set_property() calls.
                            "update_command" => i, // TODO: automatically create this function from the events present in the view (or by splitting the update() fucntion).
                            method_name => panic!("Unexpected method {}", method_name),
                        }
                    },
                    Type(_) => i,
                };
            new_items.push(new_item);
        }
        new_items.push(get_container(container_method, root_widget));
        let item = Impl(unsafety, polarity, generics, path, typ, new_items);
        ast.node = item;
        let expanded = quote! {
            #ast
        };
        expanded.parse().unwrap()
    }
    else {
        panic!("Expected impl");
    }
}

fn block_to_impl_item(tokens: Tokens) -> ImplItem {
    let implementation = quote! {
        impl Test {
            #tokens
        }
    };
    let implementation = parse_item(implementation.as_str()).unwrap();
    match implementation.node {
        Impl(_, _, _, _, _, items) => items[0].clone(),
        _ => unreachable!(),
    }
}

fn impl_view(name: &Ident, tokens: Vec<TokenTree>, root_widget: &mut Option<Ident>) -> ImplItem {
    if let TokenTree::Delimited(Delimited { ref tts, .. }) = tokens[0] {
        let widget = parse(tts);
        let view = gen(name, widget, root_widget);
        block_to_impl_item(quote! {
            #[allow(unused_variables)]
            fn view(relm: RemoteRelm<Msg>, model: &Self::Model) -> Self {
                #view
            }
        })
    }
    else {
        panic!("Expected `{{` but found `{:?}` in view! macro", tokens[0]);
    }
}

fn get_container(container_method: Option<ImplItem>, root_widget: Option<Ident>) -> ImplItem {
    container_method.unwrap_or_else(|| {
        let root_widget = root_widget.unwrap();
        block_to_impl_item(quote! {
            fn container(&self) -> &Self::Container {
                &self.#root_widget
            }
        })
    })
}

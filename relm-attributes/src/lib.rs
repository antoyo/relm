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

/*
 * TODO: allow pattern matching by creating a function update(&mut self, Quit: Msg, model: &mut Model) so that we can separate
 * the update function in multiple functions.
 */

#![feature(proc_macro)]

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod adder;
mod gen;
mod parser;

use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};
use syn::{Delimited, Expr, ExprKind, Ident, ImplItem, Mac, TokenTree, parse_expr, parse_item};
use syn::ItemKind::Impl;
use syn::ImplItemKind::{Const, Macro, Method, Type};
use syn::Ty::{self, Path};

use adder::{Property, add_set_property_to_block};
use gen::gen;
use parser::{Widget, parse};

type PropertyModelMap = HashMap<Ident, HashSet<Property>>;

#[proc_macro_attribute]
pub fn widget(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let mut ast = parse_item(&source).unwrap();
    if let Impl(unsafety, polarity, generics, path, typ, items) = ast.node {
        let name = get_name(&typ);
        let mut new_items = vec![];
        let mut container_method = None;
        let mut root_widget = None;
        let mut update_method = None;
        let mut properties_model_map = None;
        for item in items {
            let i = item.clone();
            let new_item =
                match item.node {
                    Const(_, _) => panic!("Unexpected const item"),
                    Macro(mac) => {
                        let (view, map) = get_view(mac, &name, &mut root_widget);
                        properties_model_map = Some(map);
                        view
                    },
                    Method(_, _) => {
                        match item.ident.to_string().as_ref() {
                            "container" => {
                                container_method = Some(i);
                                continue;
                            },
                            "model" => i,
                            "subscriptions" => i,
                            "update" => {
                                update_method = Some(i);
                                continue
                            },
                            "update_command" => i, // TODO: automatically create this function from the events present in the view (or by splitting the update() fucntion).
                            method_name => panic!("Unexpected method {}", method_name),
                        }
                    },
                    Type(_) => i,
                };
            new_items.push(new_item);
        }
        new_items.push(get_update(update_method.unwrap(), properties_model_map.unwrap()));
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

fn impl_view(name: &Ident, tokens: Vec<TokenTree>, root_widget: &mut Option<Ident>) -> (ImplItem, PropertyModelMap) {
    if let TokenTree::Delimited(Delimited { ref tts, .. }) = tokens[0] {
        let widget = parse(tts);
        let mut properties_model_map = HashMap::new();
        get_properties_model_map(&widget, &mut properties_model_map);
        let view = gen(name, widget, root_widget);
        let item = block_to_impl_item(quote! {
            #[allow(unused_variables)]
            fn view(relm: RemoteRelm<Msg>, model: &Self::Model) -> Self {
                #view
            }
        });
        (item, properties_model_map)
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

fn get_name(typ: &Ty) -> Ident {
    if let Path(_, ref path) = *typ {
        let mut tokens = Tokens::new();
        path.to_tokens(&mut tokens);
        Ident::new(tokens.to_string())
    }
    else {
        panic!("Expected Path")
    }
}

/*
 * TODO: Create a control flow graph for each variable of the model.
 * Add the set_property() calls in every leaf of every graphs.
 */
fn get_update(mut func: ImplItem, map: PropertyModelMap) -> ImplItem {
    if let Method(_, ref mut block) = func.node {
        add_set_property_to_block(block, &map);
    }
    // TODO: consider gtk::main_quit() as return.
    // TODO: automatically add the widget set_property() calls.
    func
}

fn get_view(mac: Mac, name: &Ident, root_widget: &mut Option<Ident>) -> (ImplItem, PropertyModelMap) {
    let segments = mac.path.segments;
    if segments.len() == 1 && segments[0].ident.to_string() == "view" {
        impl_view(name, mac.tts, root_widget)
    }
    else {
        panic!("Unexpected macro item")
    }
}

/*
 * The map maps model variable name to a vector of tuples (widget name, property name).
 */
fn get_properties_model_map(widget: &Widget, map: &mut PropertyModelMap) {
    for (name, value) in &widget.properties {
        let string = value.parse::<String>().unwrap();
        let expr = parse_expr(&string).unwrap();
        let model_variables = get_all_model_variables(&expr);
        for var in model_variables {
            let set = map.entry(var).or_insert_with(HashSet::new);
            set.insert(Property {
                expr: string.clone(),
                name: name.clone(),
                widget_name: widget.name.clone(),
            });
        }
    }
    for child in &widget.children {
        get_properties_model_map(child, map);
    }
}

fn get_all_model_variables(expr: &Expr) -> Vec<Ident> {
    let mut variables = vec![];
    match expr.node {
        ExprKind::AddrOf(_, ref expr) => {
            variables.append(&mut get_all_model_variables(&expr));
        },
        ExprKind::Field(ref expr, ref field) => {
            if let ExprKind::Path(_, ref path) = expr.node {
                if path.segments[0].ident == Ident::new("model") {
                    variables.push(field.clone());
                }
            }
            else {
                variables.append(&mut get_all_model_variables(&expr));
            }
        },
        ExprKind::Lit(_) | ExprKind::Path(_, _) => (), // No variable in these expressions.
        ExprKind::MethodCall(_, _, ref exprs) => {
            for expr in exprs {
                variables.append(&mut get_all_model_variables(expr));
            }
        },
        _ => panic!("unimplemented expr {:?}", expr.node),
    }
    variables
}

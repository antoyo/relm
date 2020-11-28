/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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

//! Transformer to transform the self.model by the actual model identifier.

use std::sync::atomic::{AtomicUsize, Ordering};

use quote::quote_spanned;
use proc_macro2::{Span, TokenStream};
use syn::{
    Expr,
    ExprField,
    ExprMacro,
    ExprPath,
    Ident,
    Macro,
    parse,
};
use syn::fold::{Fold, fold_expr};
use syn::Member::Named;

use super::parser::dummy_ident;

thread_local! {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
}

pub struct Transformer {
    model_ident: String,
    pub nested_widgets: Vec<TokenStream>,
}

impl Transformer {
    pub fn new(model_ident: &str) -> Self {
        Transformer {
            model_ident: model_ident.to_string(),
            nested_widgets: vec![],
        }
    }
}

use syn::spanned::Spanned;

impl Fold for Transformer {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Field(ExprField { ref base, ref member, .. }) => {
                if let Named(ref ident) = *member {
                    let mut is_inside_self = false;
                    if let Expr::Path(ExprPath { ref path, .. }) = **base {
                        if path.is_ident(&dummy_ident("self")) {
                            is_inside_self = true;
                        }
                    }

                    if is_inside_self {
                        if ident == "model" {
                            let model_ident = Ident::new(self.model_ident.as_str(), Span::call_site()); // TODO: check if the position is needed.
                            let tokens = quote_spanned! { expr.span() => {
                                let model = &#model_ident;
                                model
                            }};
                            return parse(tokens.into()).expect("model path");
                        }
                        else {
                            let tokens = quote_spanned! { expr.span() =>
                                #ident
                            };
                            return parse(tokens.into()).expect("self field path");
                        }
                    }
                }
            },
            Expr::Macro(ExprMacro { mac: Macro { ref path, ref tokens, .. }, .. }) => {
                if path.is_ident(&dummy_ident("view")) {
                    self.nested_widgets.push(tokens.clone());
                    let counter = COUNTER.with(|counter| {
                        counter.fetch_add(1, Ordering::SeqCst)
                    });
                    let widget_ident = Ident::new(&format!("__relm_nested_widget{}", counter), expr.span());
                    let tokens = quote_spanned! { expr.span() =>
                        #widget_ident
                    };
                    return parse(tokens.into()).expect("widget name replacement for nested view! macro");
                }
            },
            _ => (),
        }
        fold_expr(self, expr)
    }
}

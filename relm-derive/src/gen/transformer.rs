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

//! Transformer to transform the self.model by the actual model identifier.

use quote::quote_spanned;
use proc_macro2::Span;
use syn::{
    Expr,
    ExprField,
    ExprPath,
    Ident,
    Path,
    parse,
};
use syn::fold::{Fold, fold_expr};
use syn::Member::Named;

pub struct Transformer {
    model_ident: String,
}

impl Transformer {
    pub fn new(model_ident: &str) -> Self {
        Transformer {
            model_ident: model_ident.to_string(),
        }
    }
}

use syn::spanned::Spanned;

impl Fold for Transformer {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        if let Expr::Field(ExprField { ref base, ref member, .. }) = expr {
            if let Named(ref ident) = *member {
                if ident == "model" {
                    if let Expr::Path(ExprPath { path: Path { ref segments, .. }, .. }) = **base {
                        if segments.first().map(|segment| segment.value().ident.to_string()) ==
                            Some("self".to_string())
                        {
                            let model_ident = Ident::new(self.model_ident.as_str(), Span::call_site()); // TODO: check if the position is needed.
                            let tokens = quote_spanned! { expr.span() => {
                                let model = &#model_ident;
                                model
                            }};
                            return parse(tokens.into()).expect("model path");
                        }
                    }
                }
            }
        }
        fold_expr(self, expr)
    }
}

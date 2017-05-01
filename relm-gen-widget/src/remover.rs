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

use syn;
use syn::{Expr, Ident, parse_path};
use syn::fold::{Folder, noop_fold_expr};
use syn::ExprKind::{Field, Path};

use super::MODEL_IDENT;

pub struct Remover {
}

impl Remover {
    pub fn new() -> Self {
        Remover {
        }
    }
}

impl Folder for Remover {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        if let Field(ref field_expr, ref ident) = expr.node {
            if *ident == Ident::new("model") {
                if let Path(None, syn::Path { ref segments, .. }) = field_expr.node {
                    if segments.get(0).map(|segment| &segment.ident) == Some(&Ident::new("self")) {
                        let model_ident = Ident::new(MODEL_IDENT);
                        let tokens = quote! {
                            #model_ident
                        };
                        let path = parse_path(tokens.as_str()).expect("model path");
                        return Expr {
                            node: Path(None, path),
                            attrs: vec![],
                        };
                    }
                }
            }
        }
        noop_fold_expr(self, expr)
    }
}

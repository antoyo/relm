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

//! Visitor to get all the model attribute used in an expression.

use syn::{
    Expr,
    ExprField,
    ExprPath,
    Ident,
};
use syn::Member::Named;
use syn::visit::{Visit, visit_expr};

use super::parser::dummy_ident;

pub struct ModelVariableVisitor {
    pub idents: Vec<Ident>,
}

impl ModelVariableVisitor {
    pub fn new() -> Self {
        ModelVariableVisitor {
            idents: vec![],
        }
    }
}

impl<'ast> Visit<'ast> for ModelVariableVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Field(ExprField { base: ref obj, member: ref field, .. }) = *expr {
            if let Expr::Field(ExprField { base: ref expr, member: ref model_ident, .. }) = **obj {
                if let Expr::Path(ExprPath { ref path, .. }) = **expr {
                    if let Named(ref model_ident) = *model_ident {
                        if path.is_ident(&dummy_ident("self")) && model_ident == "model" {
                            if let Named(ref field) = *field {
                                self.idents.push(field.clone());
                            }
                        }
                    }
                }
            }
        }
        visit_expr(self, expr);
    }
}

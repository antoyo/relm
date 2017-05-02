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

use std::boxed;

use quote::Tokens;
use syn;
use syn::{Expr, ExprKind, Ident, Path, Stmt, parse_expr};
use syn::ExprKind::{Assign, AssignOp, Block, Field};
use syn::fold::{Folder, noop_fold_expr};
use syn::Stmt::Semi;
use syn::Unsafety::Normal;

use super::PropertyModelMap;

macro_rules! fold_assign {
    ($_self:expr, $lhs:expr, $new_assign:expr) => {{
        let mut statements = vec![];
        let new_statements =
            if let Field(ref field_expr, ref ident) = $lhs.node {
                if is_model_path(field_expr) {
                    Some(create_stmts(ident, $_self.map))
                }
                else {
                    None
                }
            }
            else {
                None
            };
        let mut new_assign = $new_assign;
        if let Some(mut stmts) = new_statements {
            let new_expr = Expr { node: new_assign, attrs: vec![] };
            statements.push(Semi(boxed::Box::new(new_expr)));
            statements.append(&mut stmts);
            new_assign = Block(Normal, syn::Block { stmts: statements })
        }
        new_assign
    }};
}

pub struct Adder<'a> {
    map: &'a PropertyModelMap,
}

impl<'a> Adder<'a> {
    pub fn new(map: &'a PropertyModelMap) -> Self {
        Adder {
            map: map,
        }
    }
}

impl<'a> Folder for Adder<'a> {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let lhs_clone =
            match expr.node {
                Assign(ref lhs, _) | AssignOp(_, ref lhs, _) => lhs.clone(),
                _ => return noop_fold_expr(self, expr),
            };
        let new_expr = noop_fold_expr(self, expr);
        let new_node = fold_assign!(self, lhs_clone, new_expr.node);
        Expr {
            node: new_node,
            attrs: new_expr.attrs.into_iter().map(|a| self.fold_attribute(a)).collect(),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Property {
    pub expr: Expr,
    pub is_relm_widget: bool,
    pub name: String,
    pub widget_name: Ident,
}

fn create_stmts(ident: &Ident, map: &PropertyModelMap) -> Vec<Stmt> {
    let mut stmts = vec![];
    if let Some(properties) = map.get(ident) {
        for property in properties {
            let widget_name = &property.widget_name;
            let prop_name = Ident::new(format!("set_{}", property.name));
            let mut tokens = Tokens::new();
            tokens.append_all(&[&property.expr]);
            let stmt =
                if property.is_relm_widget {
                    quote! {
                        { self.#widget_name.widget_mut().#prop_name(#tokens); }
                    }
                }
                else {
                    quote! {
                        { self.#widget_name.#prop_name(#tokens); }
                    }
                };
            let expr = parse_expr(&stmt.parse::<String>().expect("parse::<String>() in create_stmts"))
                .expect("parse_expr() in create_stmts");
            if let ExprKind::Block(_, ref block) = expr.node {
                stmts.push(block.stmts[0].clone());
            }
        }
    }
    stmts
}

fn is_model_path(expr: &Expr) -> bool {
    if let Field(ref expr, ref ident) = expr.node {
        if let Expr { node: ExprKind::Path(None, Path { ref segments, .. }), .. } = **expr {
            return segments.len() == 1 && segments[0].ident == Ident::new("self") && *ident == Ident::new("model");
        }
    }
    false
}

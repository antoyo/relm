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

use quote::Tokens;
use syn;
use syn::{Arm, Ident, Stmt, parse_expr};
use syn::ExprKind::{Assign, AssignOp, Block, Call, Field, Match, Path};
use syn::Stmt::{Expr, Semi};
use syn::Unsafety::Normal;

use super::PropertyModelMap;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Property {
    pub expr: String,
    pub name: String,
    pub widget_name: Ident,
}

pub fn add_set_property_to_block(block: &mut syn::Block, map: &PropertyModelMap) {
    for stmt in &mut block.stmts {
        add_to_stmt(stmt, map);
    }
}

fn add_to_arm(arm: &mut Arm, map: &PropertyModelMap) {
    if let Some(ref mut guard) = arm.guard {
        add_to_expr(guard, map);
    }
    add_to_expr(&mut arm.body, map);
}

fn add_to_expr(expr: &mut syn::Expr, map: &PropertyModelMap) {
    let expr_clone: syn::Expr = expr.clone();
    let mut new_expr = None;
    match expr.node {
        Assign(ref mut lhs, _) | AssignOp(_, ref mut lhs, _) => {
            let mut statements = vec![];
            if let Field(ref field_expr, ref ident) = lhs.node {
                if is_model_path(field_expr) {
                    statements.push(Semi(Box::new(expr_clone)));
                    statements.append(&mut create_stmts(ident, map));
                    new_expr = Some(syn::Expr { node: Block(Normal, syn::Block { stmts: statements }), attrs: vec![] });
                }
            }
        },
        Block(_, ref mut block) => add_set_property_to_block(block, map),
        Call(ref mut expr, ref mut exprs) => {
            add_to_expr(expr, map);
            for expr in exprs {
                add_to_expr(expr, map);
            }
        },
        Match(ref mut expr, ref mut arms) => {
            add_to_expr(expr, map);
            for arm in arms {
                add_to_arm(arm, map);
            }
        },
        Path(_, _) => (), // No expression.
        _ => panic!("unimplemented expr {:?}", expr.node),
    }
    if let Some(new_expr) = new_expr {
        *expr = new_expr;
    }
}

fn add_to_stmt(stmt: &mut Stmt, map: &PropertyModelMap) {
    match *stmt {
        Expr(ref mut expr) | Semi(ref mut expr) => add_to_expr(expr, map),
        _ => panic!("unimplemented stmt {:?}", stmt),
    }
}

fn create_stmts(ident: &Ident, map: &PropertyModelMap) -> Vec<Stmt> {
    let mut stmts = vec![];
    for property in &map[ident] {
        let widget_name = &property.widget_name;
        let prop_name = Ident::new(format!("set_{}", property.name));
        let mut tokens = Tokens::new();
        tokens.append(&property.expr);
        let stmt = quote! {
            { self.#widget_name.#prop_name(#tokens); }
        };
        let expr = parse_expr(&stmt.parse::<String>().unwrap()).unwrap();
        if let Block(_, ref block) = expr.node {
            stmts.push(block.stmts[0].clone());
        }
    }
    stmts
}

fn is_model_path(expr: &syn::Expr) -> bool {
    if let Path(_, ref path) = expr.node {
        return path.segments.len() == 1 && path.segments[0].ident == Ident::new("model");
    }
    false
}

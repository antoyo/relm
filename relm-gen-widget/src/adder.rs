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
use syn::{
    Expr,
    ExprAssign,
    ExprAssignOp,
    ExprBlock,
    ExprField,
    ExprPath,
    Ident,
    Path,
    Stmt,
    parse,
};
use syn::Expr::{Assign, AssignOp, Block, Field};
use syn::fold::{Fold, fold_expr};
use syn::Member::Named;
use syn::spanned::Spanned;

use super::{MsgModelMap, PropertyModelMap};

pub struct Adder<'a> {
    msg_map: &'a MsgModelMap,
    property_map: &'a PropertyModelMap,
}

impl<'a> Adder<'a> {
    pub fn new(property_map: &'a PropertyModelMap, msg_map: &'a MsgModelMap) -> Self {
        Adder {
            msg_map,
            property_map,
        }
    }
}

impl<'a> Adder<'a> {
    fn fold_assign(&self, lhs: Expr, mut new_assign: Expr) -> Expr {
        let mut statements = vec![];
        let new_statements =
            if let Field(ExprField { ref base, member: Named(ref ident), .. }) = lhs {
                if is_model_path(base) {
                    Some(create_stmts(ident, self.property_map, self.msg_map))
                }
                else {
                    None
                }
            }
            else {
                None
            };
        if let Some(mut stmts) = new_statements {
            let statement: Stmt = parse(quote! {
                #new_assign;
            }.into()).expect("expression statement");
            statements.push(statement);
            statements.append(&mut stmts);
            new_assign = parse(quote! {{
                #(#statements)*
            }}.into()).expect("statements");
        }
        new_assign
    }
}

impl<'a> Fold for Adder<'a> {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let lhs_clone =
            match expr {
                Assign(ExprAssign { ref left, .. }) | AssignOp(ExprAssignOp { ref left, .. }) => *left.clone(),
                _ => return fold_expr(self, expr),
            };
        let new_expr = fold_expr(self, expr);
        self.fold_assign(lhs_clone, new_expr)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Message {
    pub expr: Expr,
    pub name: Ident,
    pub widget_name: Ident,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Property {
    pub expr: Expr,
    pub is_relm_widget: bool,
    pub name: Ident,
    pub widget_name: Ident,
}

fn create_stmts(ident: &Ident, property_map: &PropertyModelMap, msg_map: &MsgModelMap) -> Vec<Stmt> {
    let mut stmts = vec![];
    stmts.append(&mut create_stmts_for_props(ident, property_map));
    stmts.append(&mut create_stmts_for_msgs(ident, msg_map));
    stmts
}

fn create_stmts_for_msgs(ident: &Ident, msg_map: &MsgModelMap) -> Vec<Stmt> {
    let mut stmts = vec![];
    if let Some(messages) = msg_map.get(ident) {
        for msg in messages {
            let widget_name = &msg.widget_name;
            let mut value = Tokens::new();
            value.append_all(&[&msg.expr]);
            let variant = &msg.name;
            let stmt = quote_spanned! { ident.span() =>
                { self.#widget_name.stream().emit(#variant(#value)); }
            };
            let expr: Expr = parse(stmt.into())
                .expect("parse() in create_stmts");
            if let Block(ExprBlock { ref block, .. }) = expr {
                stmts.push(block.stmts[0].clone());
            }
        }
    }
    stmts
}

fn create_stmts_for_props(ident: &Ident, property_map: &PropertyModelMap) -> Vec<Stmt> {
    let mut stmts = vec![];
    if let Some(properties) = property_map.get(ident) {
        for property in properties {
            let widget_name = &property.widget_name;
            let prop_name = Ident::new(&format!("set_{}", property.name), property.name.span());
            let mut tokens = Tokens::new();
            tokens.append_all(&[&property.expr]);
            let stmt =
                if property.is_relm_widget {
                    quote_spanned! { ident.span() =>
                        { self.#widget_name.#prop_name(#tokens); }
                    }
                }
                else {
                    quote_spanned! { ident.span() =>
                        { self.#widget_name.#prop_name(#tokens); }
                    }
                };
            let expr: Expr = parse(stmt.into()).expect("parse() in create_stmts");
            if let Block(ExprBlock { ref block, .. }) = expr {
                stmts.push(block.stmts[0].clone());
            }
        }
    }
    stmts
}

fn is_model_path(expr: &Expr) -> bool {
    if let Field(ExprField { ref base, ref member, .. }) = *expr {
        if let Expr::Path(ExprPath { path: Path { ref segments, .. }, ..}) = **base {
            if let Named(member_name) = *member {
                return segments.len() == 1 && segments[0].ident.as_ref() == "self" && member_name.as_ref() == "model";
            }
        }
    }
    false
}

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
use syn::{Arm, Expr, ExprKind, FieldValue, Ident, QSelf, Stmt, parse_expr};
use syn::fold::Folder;
use syn::Stmt::Semi;
use syn::Unsafety::Normal;

use super::PropertyModelMap;

macro_rules! fold_assign {
    ($self:expr, $lhs:expr, $new_assign:expr) => {{
        let mut statements = vec![];
        let new_statements =
            if let Field(ref field_expr, ref ident) = $lhs.node {
                if is_model_path(field_expr) {
                    Some(create_stmts(ident, $self.map))
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
    fn fold_expr(&mut self, Expr { node, attrs }: Expr) -> Expr {
        use syn::ExprKind::*;
        Expr {
            node: match node {
                Assign(lhs, rhs) => {
                    fold_assign!(self, lhs, Assign(lhs.lift(|e| self.fold_expr(e)),
                        rhs.lift(|e| self.fold_expr(e))))
                }
                AssignOp(bop, lhs, rhs) => {
                    fold_assign!(self, lhs, AssignOp(bop, lhs.lift(|e| self.fold_expr(e)),
                        rhs.lift(|e| self.fold_expr(e))))
                }

                // The rest is identical to the original implementation.
                Box(e) => Box(e.lift(|e| self.fold_expr(e))),
                InPlace(place, value) => {
                    InPlace(place.lift(|e| self.fold_expr(e)),
                    value.lift(|e| self.fold_expr(e)))
                }
                Array(array) => Array(array.lift(|e| self.fold_expr(e))),
                Call(function, args) => {
                    Call(function.lift(|e| self.fold_expr(e)),
                    args.lift(|e| self.fold_expr(e)))
                }
                MethodCall(method, tys, args) => {
                    MethodCall(self.fold_ident(method),
                    tys.lift(|t| self.fold_ty(t)),
                    args.lift(|e| self.fold_expr(e)))
                }
                Tup(args) => Tup(args.lift(|e| self.fold_expr(e))),
                Binary(bop, lhs, rhs) => {
                    Binary(bop,
                           lhs.lift(|e| self.fold_expr(e)),
                           rhs.lift(|e| self.fold_expr(e)))
                }
                Unary(uop, e) => Unary(uop, e.lift(|e| self.fold_expr(e))),
                Lit(lit) => Lit(self.fold_lit(lit)),
                Cast(e, ty) => {
                    Cast(e.lift(|e| self.fold_expr(e)),
                    ty.lift(|t| self.fold_ty(t)))
                }
                Type(e, ty) => {
                    Type(e.lift(|e| self.fold_expr(e)),
                    ty.lift(|t| self.fold_ty(t)))
                }
                If(e, if_block, else_block) => {
                    If(e.lift(|e| self.fold_expr(e)),
                    self.fold_block(if_block),
                    else_block.map(|v| v.lift(|e| self.fold_expr(e))))
                }
                IfLet(pat, expr, block, else_block) => {
                    IfLet(pat.lift(|p| self.fold_pat(p)),
                    expr.lift(|e| self.fold_expr(e)),
                    self.fold_block(block),
                    else_block.map(|v| v.lift(|e| self.fold_expr(e))))
                }
                While(e, block, label) => {
                    While(e.lift(|e| self.fold_expr(e)),
                    self.fold_block(block),
                    label.map(|i| self.fold_ident(i)))
                }
                WhileLet(pat, expr, block, label) => {
                    WhileLet(pat.lift(|p| self.fold_pat(p)),
                    expr.lift(|e| self.fold_expr(e)),
                    self.fold_block(block),
                    label.map(|i| self.fold_ident(i)))
                }
                ForLoop(pat, expr, block, label) => {
                    ForLoop(pat.lift(|p| self.fold_pat(p)),
                    expr.lift(|e| self.fold_expr(e)),
                    self.fold_block(block),
                    label.map(|i| self.fold_ident(i)))
                }
                Loop(block, label) => {
                    Loop(self.fold_block(block),
                    label.map(|i| self.fold_ident(i)))
                }
                Match(e, arms) => {
                    Match(e.lift(|e| self.fold_expr(e)),
                    arms.lift(|Arm { attrs, pats, guard, body }: Arm| {
                        Arm {
                            attrs: attrs.lift(|a| self.fold_attribute(a)),
                            pats: pats.lift(|p| self.fold_pat(p)),
                            guard: guard.map(|v| v.lift(|e| self.fold_expr(e))),
                            body: body.lift(|e| self.fold_expr(e)),
                        }
                    }))
                }
                Closure(capture_by, fn_decl, expr) => {
                    Closure(capture_by,
                            fn_decl.lift(|v| self.fold_fn_decl(v)),
                            expr.lift(|e| self.fold_expr(e)))
                }
                Block(unsafety, block) => Block(unsafety, self.fold_block(block)),
                Field(expr, name) => Field(expr.lift(|e| self.fold_expr(e)), self.fold_ident(name)),
                TupField(expr, index) => TupField(expr.lift(|e| self.fold_expr(e)), index),
                Index(expr, index) => {
                    Index(expr.lift(|e| self.fold_expr(e)),
                    index.lift(|e| self.fold_expr(e)))
                }
                Range(lhs, rhs, limits) => {
                    Range(lhs.map(|v| v.lift(|e| self.fold_expr(e))),
                    rhs.map(|v| v.lift(|e| self.fold_expr(e))),
                    limits)
                }
                Path(qself, path) => {
                    Path(qself.map(|v| noop_fold_qself(self, v)),
                    self.fold_path(path))
                }
                AddrOf(mutability, expr) => AddrOf(mutability, expr.lift(|e| self.fold_expr(e))),
                Break(label, expr) => {
                    Break(label.map(|i| self.fold_ident(i)),
                    expr.map(|v| v.lift(|e| self.fold_expr(e))))
                }
                Continue(label) => Continue(label.map(|i| self.fold_ident(i))),
                Ret(expr) => Ret(expr.map(|v| v.lift(|e| self.fold_expr(e)))),
                ExprKind::Mac(mac) => ExprKind::Mac(self.fold_mac(mac)),
                Struct(path, fields, expr) => {
                    Struct(self.fold_path(path),
                    fields.lift(|FieldValue { ident, expr, is_shorthand, attrs }: FieldValue| {
                        FieldValue {
                            ident: self.fold_ident(ident),
                            expr: self.fold_expr(expr),
                            is_shorthand: is_shorthand,
                            attrs: attrs.lift(|v| self.fold_attribute(v)),
                        }
                    }),
                    expr.map(|v| v.lift(|e| self.fold_expr(e))))
                }
                Repeat(element, number) => {
                    Repeat(element.lift(|e| self.fold_expr(e)),
                    number.lift(|e| self.fold_expr(e)))
                }
                Paren(expr) => Paren(expr.lift(|e| self.fold_expr(e))),
                Try(expr) => Try(expr.lift(|e| self.fold_expr(e))),
            },
            attrs: attrs.into_iter().map(|a| self.fold_attribute(a)).collect(),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Property {
    pub expr: String,
    pub name: String,
    pub widget_name: Ident,
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
        if let ExprKind::Block(_, ref block) = expr.node {
            stmts.push(block.stmts[0].clone());
        }
    }
    stmts
}

fn is_model_path(expr: &syn::Expr) -> bool {
    if let ExprKind::Path(_, ref path) = expr.node {
        return path.segments.len() == 1 && path.segments[0].ident == Ident::new("model");
    }
    false
}

// The following comes from the syn crate.
trait LiftOnce<T, U> {
    type Output;
    fn lift<F>(self, f: F) -> Self::Output where F: FnOnce(T) -> U;
}

impl<T, U> LiftOnce<T, U> for Box<T> {
    type Output = Box<U>;
    // Clippy false positive
    // https://github.com/Manishearth/rust-clippy/issues/1478
    #[cfg_attr(feature = "cargo-clippy", allow(boxed_local))]
    fn lift<F>(self, f: F) -> Box<U>
        where F: FnOnce(T) -> U
    {
        Box::new(f(*self))
    }
}

trait LiftMut<T, U> {
    type Output;
    fn lift<F>(self, f: F) -> Self::Output where F: FnMut(T) -> U;
}

impl<T, U> LiftMut<T, U> for Vec<T> {
    type Output = Vec<U>;
    fn lift<F>(self, f: F) -> Vec<U>
        where F: FnMut(T) -> U
    {
        self.into_iter().map(f).collect()
    }
}

fn noop_fold_qself<F: ?Sized + Folder>(folder: &mut F, QSelf { ty, position }: QSelf) -> QSelf {
    QSelf {
        ty: Box::new(folder.fold_ty(*(ty))),
        position: position,
    }
}

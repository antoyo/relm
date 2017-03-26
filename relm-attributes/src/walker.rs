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

use syn::{Arm, Expr, ExprKind, FieldValue, Ident};
use syn::visit::{Visitor, walk_opt_ident};

macro_rules! walk_list {
    ($visitor:expr, $method:ident, $list:expr $(, $extra_args:expr)*) => {
        for elem in $list {
            $visitor.$method(elem $(, $extra_args)*)
        }
    };
}

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

impl Visitor for ModelVariableVisitor {
    fn visit_expr(&mut self, expr: &Expr) {
        walk_list!(self, visit_attribute, &expr.attrs);
        match expr.node {
            ExprKind::Field(ref obj, ref field) => {
                if let ExprKind::Path(_, ref path) = obj.node {
                    if path.segments[0].ident == Ident::new("model") {
                        self.idents.push(field.clone());
                    }
                }
                else {
                    self.visit_expr(expr);
                    self.visit_ident(field);
                }
            }

            // The rest is identical to the original implementation.
            ExprKind::InPlace(ref place, ref value) => {
                self.visit_expr(place);
                self.visit_expr(value);
            }
            ExprKind::Call(ref callee, ref args) => {
                self.visit_expr(callee);
                walk_list!(self, visit_expr, args);
            }
            ExprKind::MethodCall(ref name, ref ty_args, ref args) => {
                self.visit_ident(name);
                walk_list!(self, visit_ty, ty_args);
                walk_list!(self, visit_expr, args);
            }
            ExprKind::Array(ref exprs) |
                ExprKind::Tup(ref exprs) => {
                    walk_list!(self, visit_expr, exprs);
                }
            ExprKind::Unary(_, ref operand) => {
                self.visit_expr(operand);
            }
            ExprKind::Lit(ref lit) => {
                self.visit_lit(lit);
            }
            ExprKind::Cast(ref expr, ref ty) |
                ExprKind::Type(ref expr, ref ty) => {
                    self.visit_expr(expr);
                    self.visit_ty(ty);
                }
            ExprKind::If(ref cond, ref cons, ref maybe_alt) => {
                self.visit_expr(cond);
                walk_list!(self, visit_stmt, &cons.stmts);
                if let Some(ref alt) = *maybe_alt {
                    self.visit_expr(alt);
                }
            }
            ExprKind::IfLet(ref pat, ref cond, ref cons, ref maybe_alt) => {
                self.visit_pat(pat);
                self.visit_expr(cond);
                walk_list!(self, visit_stmt, &cons.stmts);
                if let Some(ref alt) = *maybe_alt {
                    self.visit_expr(alt);
                }
            }
            ExprKind::While(ref cond, ref body, ref label) => {
                self.visit_expr(cond);
                walk_list!(self, visit_stmt, &body.stmts);
                walk_opt_ident(self, label);
            }
            ExprKind::WhileLet(ref pat, ref cond, ref body, ref label) => {
                self.visit_pat(pat);
                self.visit_expr(cond);
                walk_list!(self, visit_stmt, &body.stmts);
                walk_opt_ident(self, label);
            }
            ExprKind::ForLoop(ref pat, ref expr, ref body, ref label) => {
                self.visit_pat(pat);
                self.visit_expr(expr);
                walk_list!(self, visit_stmt, &body.stmts);
                walk_opt_ident(self, label);
            }
            ExprKind::Loop(ref body, ref label) => {
                walk_list!(self, visit_stmt, &body.stmts);
                walk_opt_ident(self, label);
            }
            ExprKind::Match(ref expr, ref arms) => {
                self.visit_expr(expr);
                for &Arm { ref attrs, ref pats, ref guard, ref body } in arms {
                    walk_list!(self, visit_attribute, attrs);
                    walk_list!(self, visit_pat, pats);
                    if let Some(ref guard) = *guard {
                        self.visit_expr(guard);
                    }
                    self.visit_expr(body);
                }
            }
            ExprKind::Closure(_, ref decl, ref expr) => {
                self.visit_fn_decl(decl);
                self.visit_expr(expr);
            }
            ExprKind::Block(_, ref block) => {
                walk_list!(self, visit_stmt, &block.stmts);
            }
            ExprKind::Binary(_, ref lhs, ref rhs) |
                ExprKind::Assign(ref lhs, ref rhs) |
                ExprKind::AssignOp(_, ref lhs, ref rhs) => {
                    self.visit_expr(lhs);
                    self.visit_expr(rhs);
                }
            ExprKind::TupField(ref obj, _) => {
                self.visit_expr(obj);
            }
            ExprKind::Index(ref obj, ref idx) => {
                self.visit_expr(obj);
                self.visit_expr(idx);
            }
            ExprKind::Range(ref maybe_start, ref maybe_end, _) => {
                if let Some(ref start) = *maybe_start {
                    self.visit_expr(start);
                }
                if let Some(ref end) = *maybe_end {
                    self.visit_expr(end);
                }
            }
            ExprKind::Path(ref maybe_qself, ref path) => {
                if let Some(ref qself) = *maybe_qself {
                    self.visit_ty(&qself.ty);
                }
                self.visit_path(path);
            }
            ExprKind::Break(ref maybe_label, ref maybe_expr) => {
                walk_opt_ident(self, maybe_label);
                if let Some(ref expr) = *maybe_expr {
                    self.visit_expr(expr);
                }
            }
            ExprKind::Continue(ref maybe_label) => {
                walk_opt_ident(self, maybe_label);
            }
            ExprKind::Ret(ref maybe_expr) => {
                if let Some(ref expr) = *maybe_expr {
                    self.visit_expr(expr);
                }
            }
            ExprKind::Mac(ref mac) => {
                self.visit_mac(mac);
            }
            ExprKind::Struct(ref path, ref fields, ref maybe_base) => {
                self.visit_path(path);
                for &FieldValue { ref ident, ref expr, .. } in fields {
                    self.visit_ident(ident);
                    self.visit_expr(expr);
                }
                if let Some(ref base) = *maybe_base {
                    self.visit_expr(base);
                }
            }
            ExprKind::Repeat(ref value, ref times) => {
                self.visit_expr(value);
                self.visit_expr(times);
            }
            ExprKind::Box(ref expr) |
                ExprKind::AddrOf(_, ref expr) |
                ExprKind::Paren(ref expr) |
                ExprKind::Try(ref expr) => {
                    self.visit_expr(expr);
                }
        }
    }
}

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
 * TODO: automatically add the model() method with a () return type when it is not found?
 * FIXME: Doing model.text.push_str() will not cause a set_text() to be added.
 * TODO: think about conditions and loops (widget-list).
 */

#![recursion_limit="128"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quote;
extern crate syn;

mod adder;
mod gen;
mod parser;
mod transformer;
mod walker;

use std::collections::{HashMap, HashSet};

use adder::{Adder, Message, Property};
use gen::gen;
pub use gen::gen_where_clause;
use parser::EitherWidget::{Gtk, Relm};
use parser::{Widget, parse};
use quote::Tokens;
use syn::{
    Delimited,
    FunctionRetTy,
    Generics,
    Ident,
    ImplItem,
    Mac,
    MethodSig,
    Path,
    TokenTree,
    parse_item,
    parse_type,
};
use syn::FnArg::Captured;
use syn::fold::Folder;
use syn::ImplItemKind::{Const, Macro, Method, Type};
use syn::ItemKind::Impl;
use syn::Pat::Wild;
use syn::Ty::{self, Tup};
use syn::visit::Visitor;
use walker::ModelVariableVisitor;

const MODEL_IDENT: &str = "__relm_model";

type MsgModelMap = HashMap<Ident, HashSet<Message>>;
type PropertyModelMap = HashMap<Ident, HashSet<Property>>;

#[derive(Debug)]
pub struct Driver {
    data_method: Option<ImplItem>,
    generic_types: Option<Generics>,
    model_type: Option<ImplItem>,
    model_param_type: Option<ImplItem>,
    msg_model_map: Option<MsgModelMap>,
    msg_type: Option<ImplItem>,
    other_methods: Vec<ImplItem>,
    properties_model_map: Option<PropertyModelMap>,
    root_method: Option<ImplItem>,
    root_type: Option<ImplItem>,
    root_widget: Option<Ident>,
    root_widget_expr: Option<Tokens>,
    root_widget_type: Option<Tokens>,
    update_method: Option<ImplItem>,
    view_macro: Option<Mac>,
    widget_model_type: Option<Ty>,
    widget_msg_type: Option<Ty>,
    widget_parent_id: Option<String>,
    widgets: HashMap<Ident, Tokens>, // Map widget ident to widget type.
}

struct View {
    container_impl: Tokens,
    item: ImplItem,
    msg_model_map: MsgModelMap,
    properties_model_map: PropertyModelMap,
    relm_widgets: HashMap<Ident, Path>,
    widget: Widget,
}

impl Driver {
    fn new() -> Self {
        Driver {
            data_method: None,
            generic_types: None,
            model_type: None,
            model_param_type: None,
            msg_model_map: None,
            msg_type: None,
            other_methods: vec![],
            properties_model_map: None,
            root_method: None,
            root_type: None,
            root_widget: None,
            root_widget_expr: None,
            root_widget_type: None,
            update_method: None,
            view_macro: None,
            widget_model_type: None,
            widget_msg_type: None,
            widget_parent_id: None,
            widgets: HashMap::new(),
        }
    }

    fn add_set_property_to_method(&self, func: &mut ImplItem) {
        if let Method(_, ref mut block) = func.node {
            let msg_map = self.msg_model_map.as_ref().expect("update method");
            let property_map = self.properties_model_map.as_ref().expect("update method");
            let mut adder = Adder::new(property_map, msg_map);
            *block = adder.fold_block(block.clone());
        }
    }

    fn add_widgets(&mut self, widget: &Widget, map: &PropertyModelMap) {
        // Only add widgets that are needed by the update() function.
        let mut to_add = false;
        for values in map.values() {
            for value in values {
                if value.widget_name == widget.name {
                    to_add = true;
                }
            }
        }
        if to_add {
            let widget_type = &widget.typ;
            let typ = quote! {
                #widget_type
            };
            self.widgets.insert(widget.name.clone(), typ);
        }
        for child in &widget.children {
            self.add_widgets(child, map);
        }
    }

    fn create_struct(&self, typ: &Ty, relm_widgets: &HashMap<Ident, Path>, generics: &Generics) -> Tokens {
        let where_clause = gen_where_clause(generics);
        let widgets = self.widgets.iter().filter(|&(ident, _)| !relm_widgets.contains_key(ident));
        let (idents, types): (Vec<_>, Vec<_>) = widgets.unzip();
        let relm_idents = relm_widgets.keys();
        let relm_types = relm_widgets.values();
        let widget_model_type = self.widget_model_type.as_ref().expect("missing model method");
        quote! {
            #[allow(dead_code, missing_docs)]
            pub struct #typ #where_clause {
                #(#idents: #types,)*
                #(#relm_idents: #relm_types,)*
                model: #widget_model_type,
            }
        }
    }

    fn gen_widget(&mut self, input: Tokens) -> Tokens {
        let source = input.to_string();
        let mut ast = parse_item(&source).expect("parse_item() in gen_widget()");
        if let Impl(unsafety, polarity, generics, path, typ, items) = ast.node {
            self.generic_types = Some(generics.clone());
            let name = get_name(&typ);
            let mut new_items = vec![];
            let mut update_items = vec![];
            for item in items {
                let mut i = item.clone();
                match item.node {
                    Const(_, _) => panic!("Unexpected const item"),
                    Macro(mac) => self.view_macro = Some(mac),
                    Method(sig, _) => {
                        match item.ident.to_string().as_ref() {
                            "parent_id" => self.data_method = Some(i),
                            "root" => self.root_method = Some(i),
                            "model" => {
                                self.widget_model_type = Some(get_return_type(sig));
                                add_model_param(&mut i, &mut self.model_param_type);
                                update_items.push(i);
                            },
                            "subscriptions" => update_items.push(i),
                            "init_view" | "on_add" => new_items.push(i),
                            "update" => {
                                self.widget_msg_type = Some(get_second_param_type(&sig));
                                self.update_method = Some(i)
                            },
                            _ => self.other_methods.push(i),
                        }
                    },
                    Type(_) => {
                        match item.ident.to_string().as_ref() {
                            "Root" => self.root_type = Some(i),
                            "Model" => self.model_type = Some(i),
                            "ModelParam" => self.model_param_type = Some(i),
                            "Msg" => self.msg_type = Some(i),
                            _ => panic!("Unexpected type item {:?}", item.ident),
                        }
                    },
                }
            }
            let view = self.get_view(&name, &typ);
            if let Some(on_add) = gen_set_child_prop_calls(&view.widget) {
                new_items.push(on_add);
            }
            self.msg_model_map = Some(view.msg_model_map);
            self.properties_model_map = Some(view.properties_model_map);
            new_items.push(view.item);
            self.widgets.insert(self.root_widget.clone().expect("root widget"),
            self.root_widget_type.clone().expect("root widget type"));
            let widget_struct = self.create_struct(&typ, &view.relm_widgets, &generics);
            new_items.push(self.get_root_type());
            if let Some(data_method) = self.get_data_method() {
                new_items.push(data_method);
            }
            new_items.push(self.get_root());
            let other_methods = self.get_other_methods(&typ, &generics);
            let update_impl = self.update_impl(&typ, &generics, update_items);
            let item = Impl(unsafety, polarity, generics, path, typ, new_items);
            ast.node = item;
            let container_impl = view.container_impl;
            quote! {
                #widget_struct
                #ast
                #container_impl
                #update_impl

                #other_methods
            }
        }
        else {
            panic!("Expected impl");
        }
    }

    fn get_data_method(&mut self) -> Option<ImplItem> {
        self.data_method.take().or_else(|| {
            if let Some(ref parent_id) = self.widget_parent_id {
                Some(block_to_impl_item(quote! {
                    fn parent_id() -> Option<&'static str> {
                        Some(#parent_id)
                    }
                }))
            }
            else {
                None
            }
        })
    }

    fn get_model_param_type(&mut self) -> ImplItem {
        self.model_param_type.take().unwrap_or_else(|| {
            block_to_impl_item(quote! {
                type ModelParam = ();
            })
        })
    }

    fn get_model_type(&mut self) -> ImplItem {
        self.model_type.take().unwrap_or_else(|| {
            let widget_model_type = self.widget_model_type.take().expect("missing model method");
            block_to_impl_item(quote! {
                type Model = #widget_model_type;
            })
        })
    }

    fn get_msg_type(&mut self) -> ImplItem {
        self.msg_type.take().unwrap_or_else(|| {
            let widget_msg_type = self.widget_msg_type.take().expect("missing update method");
            block_to_impl_item(quote! {
                type Msg = #widget_msg_type;
            })
        })
    }

    fn get_other_methods(&mut self, typ: &Ty, generics: &Generics) -> Tokens {
        let mut other_methods: Vec<_> = self.other_methods.drain(..).collect();
        let where_clause = gen_where_clause(generics);
        for method in &mut other_methods {
            self.add_set_property_to_method(method);
        }
        quote! {
            impl #generics #typ #where_clause {
                #(#other_methods)*
            }
        }
    }

    fn get_root(&mut self) -> ImplItem {
        self.root_method.take().unwrap_or_else(|| {
            let root_widget_expr = self.root_widget_expr.take().expect("root widget expr");
            block_to_impl_item(quote! {
                fn root(&self) -> Self::Root {
                    self.#root_widget_expr.clone()
                }
            })
        })
    }

    fn get_root_type(&mut self) -> ImplItem {
        self.root_type.take().unwrap_or_else(|| {
            let root_widget_type = self.root_widget_type.take().expect("root widget type");
            block_to_impl_item(quote! {
                type Root = #root_widget_type;
            })
        })
    }

    /*
     * TODO: Create a control flow graph for each variable of the model.
     * Add the set_property() calls in every leaf of every graphs.
     */
    fn get_update(&mut self) -> ImplItem {
        let mut func = self.update_method.take().expect("update method");
        self.add_set_property_to_method(&mut func);
        // TODO: consider gtk::main_quit() as return.
        func
    }

    fn get_view(&mut self, name: &Ident, typ: &Ty) -> View {
        {
            let segments = &self.view_macro.as_ref().expect("view! macro missing").path.segments;
            if segments.len() != 1 || segments[0].ident != "view" {
                panic!("Unexpected macro item")
            }
        }
        self.impl_view(name, typ)
    }

    fn impl_view(&mut self, name: &Ident, typ: &Ty) -> View {
        let tokens = &self.view_macro.take().expect("view_macro in impl_view()").tts;
        if let TokenTree::Delimited(Delimited { ref tts, .. }) = tokens[0] {
            let mut widget = parse(tts);
            if let Gtk(ref mut widget) = widget.widget {
                widget.relm_name = Some(typ.clone());
            }
            self.widget_parent_id = widget.parent_id.clone();
            let mut msg_model_map = HashMap::new();
            let mut properties_model_map = HashMap::new();
            get_properties_model_map(&widget, &mut properties_model_map);
            get_msg_model_map(&widget, &mut msg_model_map);
            self.add_widgets(&widget, &properties_model_map);
            let (view, relm_widgets, container_impl) = gen(name, &widget, self);
            let model_ident = Ident::new(MODEL_IDENT);
            let item = block_to_impl_item(quote! {
                #[allow(unused_variables)] // Necessary to avoid warnings in case the parameters are unused.
                fn view(relm: &::relm::Relm<Self>, #model_ident: Self::Model) -> Self {
                    #view
                }
            });
            View {
                container_impl,
                item,
                msg_model_map,
                properties_model_map,
                relm_widgets,
                widget,
            }
        }
        else {
            panic!("Expected `{{` but found `{:?}` in view! macro", tokens[0]);
        }
    }

    fn update_impl(&mut self, typ: &Ty, generics: &Generics, items: Vec<ImplItem>) -> Tokens {
        let where_clause = gen_where_clause(generics);

        let msg = self.get_msg_type();
        let model_param = self.get_model_param_type();
        let update = self.get_update();
        let model = self.get_model_type();
        quote! {
            impl #generics ::relm::Update for #typ #where_clause {
                #msg
                #model
                #model_param
                #update
                #(#items)*
            }
        }
    }
}

pub fn gen_widget(input: Tokens) -> Tokens {
    let mut driver = Driver::new();
    driver.gen_widget(input)
}

fn add_model_param(model_fn: &mut ImplItem, model_param_type: &mut Option<ImplItem>) {
    if let Method(ref mut method_sig, _) = model_fn.node {
        let len = method_sig.decl.inputs.len();
        if len == 0 || len == 1 {
            let type_tokens = quote! {
                &::relm::Relm<Self>
            };
            let typ = parse_type(type_tokens.as_str()).expect("Relm type");
            method_sig.decl.inputs.insert(0, Captured(Wild, typ));
            if len == 0 {
                method_sig.decl.inputs.push(Captured(Wild, Tup(vec![])));
            }
        }
        if let Some(&Captured(_, ref path)) = method_sig.decl.inputs.get(1) {
            *model_param_type = Some(block_to_impl_item(quote! {
                type ModelParam = #path;
            }));
        }
    }
}

fn block_to_impl_item(tokens: Tokens) -> ImplItem {
    let implementation = quote! {
        impl Test {
            #tokens
        }
    };
    let implementation = parse_item(implementation.as_str()).expect("parse_item in block_to_impl_item");
    match implementation.node {
        Impl(_, _, _, _, _, items) => items[0].clone(),
        _ => unreachable!(),
    }
}

fn get_name(typ: &Ty) -> Ident {
    if let Ty::Path(_, ref path) = *typ {
        let mut parts = vec![];
        for segment in &path.segments {
            parts.push(segment.ident.as_ref());
        }
        Ident::new(parts.join("::"))
    }
    else {
        panic!("Expected Path")
    }
}

macro_rules! get_map {
    ($widget:expr, $map:expr, $is_relm:expr) => {{
        for (name, expr) in &$widget.properties {
            let mut visitor = ModelVariableVisitor::new();
            visitor.visit_expr(&expr);
            let model_variables = visitor.idents;
            for var in model_variables {
                let set = $map.entry(var).or_insert_with(HashSet::new);
                set.insert(Property {
                    expr: expr.clone(),
                    is_relm_widget: $is_relm,
                    name: name.clone(),
                    widget_name: $widget.name.clone(),
                });
            }
        }
        for child in &$widget.children {
            get_properties_model_map(child, $map);
        }
    }};
}

fn get_msg_model_map(widget: &Widget, map: &mut MsgModelMap) {
    match widget.widget {
        Gtk(_) => {
            for child in &widget.children {
                get_msg_model_map(child, map);
            }
        },
        Relm(ref relm_widget) => {
            for (name, expr) in &relm_widget.messages {
                let mut visitor = ModelVariableVisitor::new();
                visitor.visit_expr(&expr);
                let model_variables = visitor.idents;
                for var in model_variables {
                    let set = map.entry(var).or_insert_with(HashSet::new);
                    set.insert(Message {
                        expr: expr.clone(),
                        name: name.clone(),
                        widget_name: widget.name.clone(),
                    });
                }
            }
            for child in &widget.children {
                get_msg_model_map(child, map);
            }
        },
    }
}

/*
 * The map maps model variable name to a vector of tuples (widget name, property name).
 */
fn get_properties_model_map(widget: &Widget, map: &mut PropertyModelMap) {
    match widget.widget {
        Gtk(_) => get_map!(widget, map, false),
        Relm(_) => get_map!(widget, map, true),
    }
}

fn get_return_type(sig: MethodSig) -> Ty {
    if let FunctionRetTy::Ty(ty) = sig.decl.output {
        ty
    }
    else {
        panic!("Unexpected default, expecting Ty");
    }
}

fn get_second_param_type(sig: &MethodSig) -> Ty {
    if let Captured(_, ref path) = sig.decl.inputs[1] {
        path.clone()
    }
    else {
        panic!("Unexpected `{:?}`, expecting Captured Ty", sig.decl.inputs[1]);
    }
}

fn gen_set_child_prop_calls(widget: &Widget) -> Option<ImplItem> {
    let mut tokens = Tokens::new();
    let widget_name = &widget.name;
    for (key, value) in &widget.child_properties {
        let property_func = Ident::new(format!("set_child_{}", key));
        tokens.append(quote! {
            parent.#property_func(&self.#widget_name, #value);
        });
    }
    if !widget.child_properties.is_empty() {
        Some(block_to_impl_item(quote! {
            fn on_add<W: ::gtk::IsA<::gtk::Widget> + ::gtk::IsA<::gtk::Object>>(&self, parent: W) {
                let parent: gtk::Box = ::gtk::Cast::downcast(::gtk::Cast::upcast::<::gtk::Widget>(parent))
                    .expect("the parent of a widget with child properties must be a gtk::Box");
                #tokens
            }
        }))
    }
    else {
        None
    }
}

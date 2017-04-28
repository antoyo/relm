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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quote;
extern crate syn;

mod adder;
mod gen;
mod parser;
mod walker;

use std::collections::{HashMap, HashSet};

use adder::{Adder, Property};
use gen::gen;
use parser::Widget::{Gtk, Relm};
use parser::{Widget, parse};
use quote::Tokens;
use syn::{
    AngleBracketedParameterData,
    Delimited,
    FunctionRetTy,
    Generics,
    Ident,
    ImplItem,
    Mac,
    MethodSig,
    Path,
    PathSegment,
    TokenTree,
    parse_expr,
    parse_item,
};
use syn::FnArg::Captured;
use syn::fold::Folder;
use syn::ImplItemKind::{Const, Macro, Method, Type};
use syn::ItemKind::Impl;
use syn::Pat::Wild;
use syn::PathParameters::AngleBracketed;
use syn::Ty::{self, Tup};
use syn::visit::Visitor;
use walker::ModelVariableVisitor;

type PropertyModelMap = HashMap<Ident, HashSet<Property>>;

// TODO: create a struct for this module instead of having to carry around a State.
#[derive(Debug)]
struct State {
    generic_types: Generics,
    model_type: Option<ImplItem>,
    model_param_type: Option<ImplItem>,
    msg_type: Option<ImplItem>,
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
    widgets: HashMap<Ident, Tokens>, // Map widget ident to widget type.
}

struct View {
    container_impl: Tokens,
    item: ImplItem,
    properties_model_map: PropertyModelMap,
    relm_widgets: HashMap<Ident, Path>,
    widget: Widget,
}

impl State {
    fn new(generics: Generics) -> Self {
        State {
            generic_types: generics,
            root_method: None,
            root_type: None,
            model_type: None,
            model_param_type: None,
            msg_type: None,
            properties_model_map: None,
            root_widget: None,
            root_widget_expr: None,
            root_widget_type: None,
            update_method: None,
            view_macro: None,
            widget_model_type: None,
            widget_msg_type: None,
            widgets: HashMap::new(),
        }
    }
}

pub fn gen_widget(input: Tokens) -> Tokens {
    let source = input.to_string();
    let mut ast = parse_item(&source).expect("parse_item() in gen_widget()");
    if let Impl(unsafety, polarity, generics, path, typ, items) = ast.node {
        let name = get_name(&typ);
        let mut new_items = vec![];
        let mut state = State::new(generics.clone());
        for item in items {
            let mut i = item.clone();
            match item.node {
                Const(_, _) => panic!("Unexpected const item"),
                Macro(mac) => state.view_macro = Some(mac),
                Method(sig, _) => {
                    match item.ident.to_string().as_ref() {
                        "root" => state.root_method = Some(i),
                        "model" => {
                            state.widget_model_type = Some(get_return_type(sig));
                            add_model_param(&mut i, &mut state.model_param_type);
                            new_items.push(i);
                        },
                        "init_view" | "subscriptions" | "update_command" => new_items.push(i),
                        "update" => {
                            state.widget_msg_type = Some(get_second_param_type(&sig));
                            state.update_method = Some(i)
                        },
                        method_name => panic!("Unexpected method {}", method_name),
                    }
                },
                Type(_) => {
                    match item.ident.to_string().as_ref() {
                        "Root" => state.root_type = Some(i),
                        "Model" => state.model_type = Some(i),
                        "ModelParam" => state.model_param_type = Some(i),
                        "Msg" => state.msg_type = Some(i),
                        _ => panic!("Unexpected type item {:?}", item.ident),
                    }
                },
            }
        }
        let view = get_view(&name, &typ, &mut state);
        if let Some(on_add) = gen_set_child_prop_calls(&view.widget) {
            new_items.push(on_add);
        }
        state.properties_model_map = Some(view.properties_model_map);
        new_items.push(view.item);
        state.widgets.insert(state.root_widget.clone().expect("root widget"),
            state.root_widget_type.clone().expect("root widget type"));
        new_items.push(get_msg_type(state.msg_type, state.widget_msg_type));
        new_items.push(get_model_type(state.model_type, state.widget_model_type));
        new_items.push(get_model_param_type(state.model_param_type));
        new_items.push(get_root_type(state.root_type, state.root_widget_type));
        new_items.push(get_update(state.update_method.expect("update method"),
            &state.properties_model_map.expect("properties model map")));
        new_items.push(get_root(state.root_method, state.root_widget_expr));
        let widget_struct = create_struct(&typ, &state.widgets, &view.relm_widgets);
        let item = Impl(unsafety, polarity, generics, path, typ, new_items);
        ast.node = item;
        let container_impl = view.container_impl;
        quote! {
            #widget_struct
            #ast
            #container_impl
        }
    }
    else {
        panic!("Expected impl");
    }
}

fn add_model_param(model_fn: &mut ImplItem, model_param_type: &mut Option<ImplItem>) {
    if let Method(ref mut method_sig, _) = model_fn.node {
        if method_sig.decl.inputs.is_empty() {
            method_sig.decl.inputs.push(Captured(Wild, Tup(vec![])));
        }
        else {
            if let Captured(_, ref path) = method_sig.decl.inputs[0] {
                *model_param_type = Some(block_to_impl_item(quote! {
                    type ModelParam = #path;
                }));
            }
        }
    }
}

fn add_widgets(widget: &Widget, widgets: &mut HashMap<Ident, Tokens>, map: &PropertyModelMap) {
    // Only add widgets that are needed by the update() function.
    let mut to_add = false;
    for values in map.values() {
        for value in values {
            if value.widget_name == widget.name() {
                to_add = true;
            }
        }
    }
    if to_add {
        let widget_type = widget.typ();
        let typ = quote! {
            #widget_type
        };
        widgets.insert(widget.name().clone(), typ);
    }
    match *widget {
        Gtk(ref widget) =>  {
            for child in &widget.children {
                add_widgets(child, widgets, map);
            }
        },
        Relm(ref widget) =>  {
            for child in &widget.children {
                add_widgets(child, widgets, map);
            }
        },
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

fn create_struct(typ: &Ty, widgets: &HashMap<Ident, Tokens>, relm_widgets: &HashMap<Ident, Path>) -> Tokens {
    let widgets = widgets.iter().filter(|&(ident, _)| !relm_widgets.contains_key(ident));
    let (idents, types): (Vec<_>, Vec<_>) = widgets.unzip();
    let relm_idents = relm_widgets.keys();
    let relm_types = relm_widgets.values();
    let phantom_field = get_phantom_field(typ);
    quote! {
        #[allow(dead_code)]
        #[derive(ManualClone)]
        pub struct #typ {
            #(#idents: #types,)*
            #(#relm_idents: #relm_types,)*
            #phantom_field
        }
    }
}

fn get_model_type(model_type: Option<ImplItem>, widget_model_type: Option<Ty>) -> ImplItem {
    model_type.unwrap_or_else(|| {
        let widget_model_type = widget_model_type.expect("missing model method");
        block_to_impl_item(quote! {
            type Model = #widget_model_type;
        })
    })
}

fn get_model_param_type(model_param_type: Option<ImplItem>) -> ImplItem {
    model_param_type.unwrap_or_else(|| {
        block_to_impl_item(quote! {
            type ModelParam = ();
        })
    })
}

fn get_msg_type(msg_type: Option<ImplItem>, widget_msg_type: Option<Ty>) -> ImplItem {
    msg_type.unwrap_or_else(|| {
        let widget_msg_type = widget_msg_type.expect("missing update method");
        block_to_impl_item(quote! {
            type Msg = #widget_msg_type;
        })
    })
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

fn get_generic_types(typ: &Ty) -> Option<Vec<Ident>> {
    if let Ty::Path(_, ref path) = *typ {
        let last_segment = path.segments.last().expect("path should have at least one segment");
        if let PathSegment {
                parameters: AngleBracketed(AngleBracketedParameterData {
                    ref types, ..
                }), ..
            } = *last_segment
        {
            let mut generic_types = vec![];
            for typ in types {
                if let &Ty::Path(_, Path { ref segments, .. }) = typ {
                    if let Some(&PathSegment { ref ident, .. }) = segments.first() {
                        generic_types.push(ident.clone());
                    }
                }
            }
            if !generic_types.is_empty() {
                return Some(generic_types);
            }
        }
    }
    None
}

fn get_phantom_field(typ: &Ty) -> Tokens {
    if let Some(types) = get_generic_types(typ) {
        let fields = types.iter().map(|typ| {
            let name = Ident::new(format!("__relm_phantom_marker_{}", typ.as_ref().to_lowercase()));
            quote! {
                #name: ::std::marker::PhantomData<#typ>,
            }
        });
        quote! {
            #(#fields)*
        }
    }
    else {
        quote! {
        }
    }
}

macro_rules! get_map {
    ($widget:expr, $map:expr, $is_relm:expr) => {{
        for (name, value) in &$widget.properties {
            let string = value.parse::<String>().expect("parse::<String>() in get_map!");
            let expr = parse_expr(&string).expect("parse_expr in get_map!");
            let mut visitor = ModelVariableVisitor::new();
            visitor.visit_expr(&expr);
            let model_variables = visitor.idents;
            for var in model_variables {
                let set = $map.entry(var).or_insert_with(HashSet::new);
                set.insert(Property {
                    expr: string.clone(),
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

/*
 * The map maps model variable name to a vector of tuples (widget name, property name).
 */
fn get_properties_model_map(widget: &Widget, map: &mut PropertyModelMap) {
    match *widget {
        Gtk(ref widget) => get_map!(widget, map, false),
        Relm(ref widget) => get_map!(widget, map, true),
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

fn get_root(root_method: Option<ImplItem>, root_widget_expr: Option<Tokens>) -> ImplItem {
    root_method.unwrap_or_else(|| {
        let root_widget_expr = root_widget_expr.expect("root widget expr");
        block_to_impl_item(quote! {
            fn root(&self) -> &Self::Root {
                &self.#root_widget_expr
            }
        })
    })
}

fn get_root_type(root_type: Option<ImplItem>, root_widget_type: Option<Tokens>) -> ImplItem {
    root_type.unwrap_or_else(|| {
        let root_widget_type = root_widget_type.expect("root widget type");
        block_to_impl_item(quote! {
            type Root = #root_widget_type;
        })
    })
}

fn get_second_param_type(sig: &MethodSig) -> Ty {
    if let Captured(_, ref path) = sig.decl.inputs[1] {
        path.clone()
    }
    else {
        panic!("Unexpected `{:?}`, expecting Captured Ty", sig.decl.inputs[1]);
    }
}

/*
 * TODO: Create a control flow graph for each variable of the model.
 * Add the set_property() calls in every leaf of every graphs.
 */
fn get_update(mut func: ImplItem, map: &PropertyModelMap) -> ImplItem {
    if let Method(_, ref mut block) = func.node {
        let mut adder = Adder::new(map);
        *block = adder.fold_block(block.clone());
    }
    // TODO: consider gtk::main_quit() as return.
    func
}

fn get_view(name: &Ident, typ: &Ty, state: &mut State) -> View {
    {
        let segments = &state.view_macro.as_ref().expect("view! macro missing").path.segments;
        if segments.len() != 1 || segments[0].ident != "view" {
            panic!("Unexpected macro item")
        }
    }
    impl_view(name, typ, state)
}

fn impl_view(name: &Ident, typ: &Ty, state: &mut State) -> View {
    let tokens = &state.view_macro.as_ref().expect("view_macro in impl_view()").tts;
    if let TokenTree::Delimited(Delimited { ref tts, .. }) = tokens[0] {
        let mut widget = parse(tts);
        if let Gtk(ref mut widget) = widget {
            widget.relm_name = Some(typ.clone());
        }
        let mut properties_model_map = HashMap::new();
        get_properties_model_map(&widget, &mut properties_model_map);
        add_widgets(&widget, &mut state.widgets, &properties_model_map);
        let idents: Vec<_> = state.widgets.keys().collect();
        let (view, relm_widgets, container_impl) =
            gen(name, typ, &widget, &mut state.root_widget, &mut state.root_widget_expr,
                &mut state.root_widget_type, &idents, &state.generic_types);
        let item = block_to_impl_item(quote! {
            #[allow(unused_variables)] // Necessary to avoid warnings in case the parameters are unused.
            fn view(relm: &::relm::RemoteRelm<Self>, model: &Self::Model) -> Self {
                #view
            }
        });
        View {
            container_impl: container_impl,
            item: item,
            properties_model_map: properties_model_map,
            relm_widgets: relm_widgets,
            widget: widget,
        }
    }
    else {
        panic!("Expected `{{` but found `{:?}` in view! macro", tokens[0]);
    }
}

fn gen_set_child_prop_calls(widget: &Widget) -> Option<ImplItem> {
    let widget = match *widget {
        Gtk(ref gtk_widget) => gtk_widget,
        Relm(_) => return None,
    };
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

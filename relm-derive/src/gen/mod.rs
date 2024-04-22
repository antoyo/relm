/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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

pub(crate) mod parser;

mod adder;
mod generator;
mod transformer;
mod walker;

use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    Error,
    Generics,
    Ident,
    ImplItem,
    ImplItemMethod,
    ImplItemType,
    ItemImpl,
    Macro,
    Path,
    PatType,
    PathArguments,
    ReturnType,
    Signature,
    TypePath,
    parse,
};
use syn::FnArg::{self, Typed};
use syn::fold::Fold;
use syn::ImplItem::Method;
use syn::Item::{self, Impl};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::Type;
use syn::visit::Visit;

use self::adder::{Adder, Message, Property};
pub use self::generator::gen_where_clause;
use self::parser::EitherWidget::{Gtk, Relm};
use self::parser::{Widget, WidgetList};
use self::walker::ModelVariableVisitor;

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
    root_widget_expr: Option<TokenStream>,
    root_widget_is_relm: bool,
    root_widget_type: Option<TokenStream>,
    update_method: Option<ImplItem>,
    view_macro: Option<Macro>,
    widget_model_type: Option<Type>,
    widget_msg_type: Option<Type>,
    widget_parent_id: Option<String>,
    widgets: HashMap<Ident, TokenStream>, // Map widget ident to widget type.
}

pub struct WidgetDefinition {
    impl_item: ItemImpl,
    methods: WidgetMethods,
    types: WidgetTypes,
    view: WidgetList,
}

pub struct WidgetMethods {
    model: ImplItemMethod,
    update: ImplItemMethod,
    parent_id: Option<ImplItemMethod>,
    root: Option<ImplItemMethod>,
    subscriptions: Option<ImplItemMethod>,
    init_view: Option<ImplItemMethod>,
    on_add: Option<ImplItemMethod>,
    other: Vec<ImplItemMethod>,
}

pub struct WidgetTypes {
    root: Option<ImplItemType>,
    model: Option<ImplItemType>,
    model_param: Option<ImplItemType>,
    msg: Option<ImplItemType>,
}

impl Parse for WidgetDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ast: ItemImpl = input.parse()?;

        let mut model_method = None;
        let mut update_method = None;
        let mut parent_id_method = None;
        let mut root_method = None;
        let mut subscriptions_method = None;
        let mut init_view_method = None;
        let mut on_add_method = None;
        let mut other_methods = vec![];

        let mut root_type = None;
        let mut model_type = None;
        let mut model_param_type = None;
        let mut msg_type = None;

        let mut widget_list = None;

        for item in ast.items.drain(..) {
            match item {
                Method(meth) => {
                    match &*meth.sig.ident.to_string() {
                        "model" => model_method = Some(meth),
                        "root" => root_method = Some(meth),
                        "update" => update_method = Some(meth),
                        "parent_id" => parent_id_method = Some(meth),
                        "subscriptions" => subscriptions_method = Some(meth),
                        "init_view" => init_view_method = Some(meth),
                        "on_add" => on_add_method = Some(meth),
                        _ => other_methods.push(meth),
                    }
                },
                ImplItem::Macro(mac) => widget_list = Some(mac.mac.parse_body()?),
                ImplItem::Type(ty) => {
                    match &*ty.ident.to_string() {
                        "Root" => root_type = Some(ty),
                        "Model" => model_type = Some(ty),
                        "ModelParam" => model_param_type = Some(ty),
                        "Msg" => msg_type = Some(ty),
                        _ => return Err(Error::new(ty.span(), "unexpected type")),
                    }
                }
                item => return Err(Error::new(item.span(), "unexpected item")),
            }
        }

        Ok(WidgetDefinition {
            impl_item: ast,
            methods: WidgetMethods {
                model: model_method.ok_or_else(|| input.error("model method not found"))?,
                update: update_method.ok_or_else(|| input.error("update method not found"))?,
                parent_id: parent_id_method,
                root: root_method,
                subscriptions: subscriptions_method,
                init_view: init_view_method,
                on_add: on_add_method,
                other: other_methods,
            },
            types: WidgetTypes {
                root: root_type,
                model: model_type,
                model_param: model_param_type,
                msg: msg_type,
            },
            view: widget_list.ok_or_else(|| input.error("view macro not found"))?,
        })
    }
}

struct View {
    container_impl: TokenStream,
    item: ImplItem,
    msg_model_map: MsgModelMap,
    properties_model_map: PropertyModelMap,
    relm_components: HashMap<Ident, Path>,
    relm_widgets: HashMap<Ident, Path>,
    streams_to_save: HashSet<Ident>,
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
            root_widget_is_relm: false,
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
        if let Method(ImplItemMethod { ref mut block, .. }) = *func {
            let msg_map = self.msg_model_map.as_ref().expect("update method");
            let property_map = self.properties_model_map.as_ref().expect("update method");
            let mut adder = Adder::new(property_map, msg_map);
            *block = adder.fold_block(block.clone());
        }
    }

    fn collect_bindings(&mut self, widget: &Widget, msg_model_map: &mut MsgModelMap, properties_model_map: &mut PropertyModelMap) {
        get_properties_model_map(widget, properties_model_map);
        get_msg_model_map(widget, msg_model_map);
        self.add_widgets(widget, properties_model_map);

        for nested_view in widget.nested_views.values() {
            self.collect_bindings(nested_view, msg_model_map, properties_model_map);
        }

        for child in &widget.children {
            self.collect_bindings(child, msg_model_map, properties_model_map);
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
    }

    fn create_struct(&self, typ: &Type, relm_widgets: &HashMap<Ident, Path>, relm_components: &HashMap<Ident, Path>, streams_to_save: &HashSet<Ident>, generics: &Generics) -> TokenStream {
        let where_clause = gen_where_clause(generics);
        let root_widget_name = self.root_widget.as_ref().expect("root widget name");
        let widgets = self.widgets.iter()
            .filter(|&(ident, _)| !relm_widgets.contains_key(ident) && !relm_components.contains_key(ident) && ident != root_widget_name)
            .map(|(ident, tokens)| (ident.clone(), tokens));
        let (idents, types): (Vec<Ident>, Vec<_>) = widgets.unzip();
        let widget_model_type = self.widget_model_type.as_ref().expect("missing model method");
        let components_name = Ident::new(&format!("__{}Components", get_name(typ)), Span::call_site());
        let widgets_name = Ident::new(&format!("__{}Widgets", get_name(typ)), Span::call_site());
        let streams_name = Ident::new(&format!("__{}Streams", get_name(typ)), Span::call_site());
        let components = {
            let components = relm_components.iter()
                .map(|(ident, tokens)| (ident.clone(), tokens));
            let (idents, types): (Vec<Ident>, Vec<_>) = components.unzip();
            quote! {
                pub struct #components_name {
                    #(pub #idents: #types,)*
                }
            }
        };

        let component_root_types = relm_components.values();
        let component_root_types: Vec<_> = component_root_types
            .map(|path| {
                if let PathArguments::AngleBracketed(ref arguments) = path.segments.last().expect("component").arguments {
                    let first_arg = arguments.args.first();
                    let arg = first_arg.as_ref().expect("argument");
                    return *arg;
                }
                panic!("Not a component type");
            })
            .collect();
        let widgets = {
            let relm_idents = relm_widgets.keys();
            let relm_types = relm_widgets.values();

            let component_idents = relm_components.keys();
            quote! {
                #[derive(Clone)]
                pub struct #widgets_name {
                    #(#component_idents: <#component_root_types as ::relm::Widget>::Root,)*
                    #(#idents: #types,)*
                    #(#relm_idents: #relm_types,)*
                }
            }
        };
        let streams = {
            let (component_idents, component_root_types): (Vec<_>, Vec<_>) = relm_components.iter()
                .filter(|(ident, _)| streams_to_save.contains(ident))
                .map(|(ident, path)| {
                    if let PathArguments::AngleBracketed(ref arguments) = path.segments.last().expect("component").arguments {
                        let first_arg = arguments.args.first();
                        let arg = first_arg.as_ref().expect("argument");
                        return (ident, *arg);
                    }
                    panic!("Not a component type");
                })
                .unzip();
            quote! {
                #[derive(Clone)]
                pub struct #streams_name {
                    #(#component_idents: ::relm::StreamHandle<<#component_root_types as ::relm::Update>::Msg>,)*
                }
            }
        };
        quote_spanned! { typ.span() =>
            #[allow(dead_code, missing_docs)]
            pub struct #typ #where_clause {
                streams: #streams_name,
                components: #components_name,
                widgets: #widgets_name,
                model: #widget_model_type,
            }

            #components

            #streams

            #widgets
        }
    }

    fn gen_widget(&mut self, definition: WidgetDefinition) -> TokenStream {
        let ItemImpl { generics, self_ty, .. } = &definition.impl_item;

        self.generic_types = Some(generics.clone());

        let mut update_items: Vec<ImplItem> = vec![];
        let mut new_items: Vec<ImplItem> = vec![];

        self.root_type = definition.types.root.map(ImplItem::Type);
        self.model_type = definition.types.model.map(ImplItem::Type);
        self.model_param_type = definition.types.model_param.map(ImplItem::Type);
        self.msg_type = definition.types.msg.map(ImplItem::Type);

        self.data_method = definition.methods.parent_id.map(ImplItem::Method);
        self.root_method = definition.methods.root.map(ImplItem::Method);

        self.widget_model_type = Some(get_return_type(definition.methods.model.sig.clone()));
        let mut model_method_copy = definition.methods.model.clone();
        add_model_param(&mut model_method_copy, &mut self.model_param_type);
        update_items.push(ImplItem::Method(model_method_copy));

        if let Some(subscriptions_method) = definition.methods.subscriptions {
            update_items.push(ImplItem::Method(subscriptions_method));
        }

        if let Some(init_view_method) = definition.methods.init_view {
            new_items.push(ImplItem::Method(init_view_method))
        }

        if let Some(on_add_method) = definition.methods.on_add {
            new_items.push(ImplItem::Method(on_add_method));
        }

        self.other_methods = definition.methods.other.into_iter().map(ImplItem::Method).collect();

        self.widget_msg_type = Some(get_second_param_type(&definition.methods.update.sig));
        self.update_method = Some(ImplItem::Method(definition.methods.update));

        let name = get_name(&self_ty);
        let view = self.get_view(definition.view, &name, self_ty);

        if let Some(on_add) = gen_set_child_prop_calls(&view.widget) {
            new_items.push(on_add);
        }
        self.msg_model_map = Some(view.msg_model_map);
        self.properties_model_map = Some(view.properties_model_map);
        new_items.push(view.item);
        self.widgets.insert(self.root_widget.clone().expect("root widget"),
        self.root_widget_type.clone().expect("root widget type"));
        let widget_struct = self.create_struct(&self_ty, &view.relm_widgets, &view.relm_components, &view.streams_to_save, &generics);
        new_items.push(self.get_root_type());
        if let Some(data_method) = self.get_data_method() {
            new_items.push(data_method);
        }
        new_items.push(self.get_root());
        let other_methods = self.get_other_methods(&self_ty, &generics);
        let update_impl = self.update_impl(&self_ty, &generics, update_items);
        let widget_test_impl = self.widget_test_impl(&self_ty, &generics);
        let item = Impl(ItemImpl { items: new_items, ..definition.impl_item });
        let container_impl = view.container_impl;
        quote! {
            #widget_struct
            #item
            #container_impl
            #update_impl
            #widget_test_impl

            #other_methods
        }
    }

    fn get_data_method(&mut self) -> Option<ImplItem> {
        self.data_method.take().or_else(|| {
            self.widget_parent_id.as_ref().map(|parent_id| block_to_impl_item(quote! {
                    fn parent_id() -> Option<&'static str> {
                        Some(#parent_id)
                    }
                }))
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

    fn get_other_methods(&mut self, typ: &Type, generics: &Generics) -> TokenStream {
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
                    self.widgets.#root_widget_expr.clone()
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

    fn get_view(&mut self, WidgetList { mut widgets }: WidgetList, name: &Ident, typ: &Type) -> View {
        self.widget_parent_id = widgets[0].parent_id.clone();

        let mut msg_model_map = HashMap::new();
        let mut properties_model_map = HashMap::new();
        if let Gtk(ref mut widget) = widgets[0].widget {
            widget.relm_name = Some(typ.clone());
        }
        for widget in &widgets {
            self.collect_bindings(widget, &mut msg_model_map, &mut properties_model_map);
        }

        let generator::Gen { view, relm_widgets, relm_components, streams_to_save, container_impl } = generator::gen(name, &widgets, self);
        let model_ident = Ident::new(MODEL_IDENT, Span::call_site()); // TODO: maybe need to set Span here.
        let code = quote_spanned! { name.span() =>
            #[allow(unused_variables,clippy::all)] // Necessary to avoid warnings in case the parameters are unused.
            fn view(relm: &::relm::Relm<Self>, #model_ident: Self::Model) -> Self {
                #view
            }
        };
        let item = block_to_impl_item(code);
        let widget = widgets.drain(..).next().expect("first widget");
        View {
            container_impl,
            item,
            msg_model_map,
            properties_model_map,
            relm_components,
            relm_widgets,
            streams_to_save,
            widget,
        }
    }

    fn update_impl(&mut self, typ: &Type, generics: &Generics, items: Vec<ImplItem>) -> TokenStream {
        let where_clause = gen_where_clause(generics);

        let msg = self.get_msg_type();
        let model_param = self.get_model_param_type();
        let update = self.get_update();
        let model = self.get_model_type();
        quote_spanned! { typ.span() =>
            impl #generics ::relm::Update for #typ #where_clause {
                #msg
                #model
                #model_param
                #update
                #(#items)*
            }
        }
    }

    fn widget_test_impl(&self, typ: &Type, generics: &Generics) -> TokenStream {
        let streams_name = Ident::new(&format!("__{}Streams", get_name(typ)), Span::call_site());
        let name = Ident::new(&format!("__{}Widgets", get_name(typ)), Span::call_site());
        let where_clause = gen_where_clause(generics);
        quote_spanned! { typ.span() =>
            #[cfg(test)]
            impl #generics ::relm::WidgetTest for #typ #where_clause {
                type Streams = #streams_name;
                type Widgets = #name;

                fn get_streams(&self) -> #streams_name {
                    self.streams.clone()
                }

                fn get_widgets(&self) -> #name {
                    self.widgets.clone()
                }
            }
        }
    }
}

pub fn gen_widget(definition: WidgetDefinition) -> TokenStream {
    let mut driver = Driver::new();
    driver.gen_widget(definition)
}

fn add_model_param(model_fn: &mut ImplItemMethod, model_param_type: &mut Option<ImplItem>) {
    let span = model_fn.span();

    let len = model_fn.sig.inputs.len();
    if len == 0 || len == 1 {
        let type_tokens = quote_spanned! { span =>
            &::relm::Relm<Self>
        };
        let ty: Type = parse(type_tokens.into()).expect("Relm type");
        let input: FnArg = parse(quote! { _: #ty }.into()).expect("wild arg");
        model_fn.sig.inputs.insert(0, input);
        if len == 0 {
            let input: FnArg = parse(quote! { _: () }.into()).expect("wild arg");
            model_fn.sig.inputs.push(input);
        }
    }
    if let Some(&Typed(PatType { ref ty, .. })) = model_fn.sig.inputs.iter().nth(1) {
        *model_param_type = Some(block_to_impl_item(quote! {
            type ModelParam = #ty;
        }));
    }
}

fn block_to_impl_item(tokens: TokenStream) -> ImplItem {
    let implementation = quote! {
        impl Test {
            #tokens
        }
    };
    let implementation: Item = parse(implementation.into()).expect("parse_item in block_to_impl_item");
    match implementation {
        Impl(ItemImpl { items, .. }) => items[0].clone(),
        _ => unreachable!(),
    }
}

fn get_name(typ: &Type) -> Ident {
    if let Type::Path(TypePath { ref path, .. }) = *typ {
        let mut parts = vec![];
        for segment in &path.segments {
            parts.push(segment.ident.to_string());
        }
        Ident::new(&parts.join("::"), typ.span())
    }
    else {
        panic!("Expected Path")
    }
}

fn get_msg_model_map(widget: &Widget, map: &mut MsgModelMap) {
    match widget.widget {
        Gtk(_) => (),
        Relm(ref relm_widget) => {
            for (name, expr) in &relm_widget.messages {
                let mut visitor = ModelVariableVisitor::new();
                visitor.visit_expr(expr);
                let model_variables = visitor.idents;
                for var in model_variables {
                    let set = map.entry(var).or_default();
                    set.insert(Message {
                        expr: expr.clone(),
                        name: name.clone(),
                        widget_name: widget.name.clone(),
                    });
                }
            }
        },
    }
}

/*
 * The map maps model variable name to a vector of tuples (widget name, property name).
 */
fn get_properties_model_map(widget: &Widget, map: &mut PropertyModelMap) {
    match widget.widget {
        Gtk(_) => get_map(widget, map, false),
        Relm(_) => get_map(widget, map, true),
    }
}

fn get_map(widget: &Widget, map: &mut PropertyModelMap, is_relm: bool) {
    for (name, expr) in &widget.properties {
        let mut visitor = ModelVariableVisitor::new();
        visitor.visit_expr(expr);
        let model_variables = visitor.idents;
        for var in model_variables {
            let set = map.entry(var).or_default();
            set.insert(Property {
                expr: expr.clone(),
                is_relm_widget: is_relm,
                name: name.clone(),
                widget_name: widget.name.clone(),
            });
        }
    }
}

fn get_return_type(sig: Signature) -> Type {
    if let ReturnType::Type(_, ty) = sig.output {
        *ty
    }
    else {
        Type::Tuple(syn::TypeTuple {
            paren_token: syn::token::Paren::default(),
            elems: syn::punctuated::Punctuated::new()
        })
    }
}

fn get_second_param_type(sig: &Signature) -> Type {
    if let Typed(PatType { ref ty, .. }) = sig.inputs[1] {
        *ty.clone()
    }
    else {
        panic!("Unexpected `(unknown)`, expecting Typed Type"/*, sig.decl.inputs[1]*/); // TODO
    }
}

fn gen_set_child_prop_calls(widget: &Widget) -> Option<ImplItem> {
    let mut tokens = quote! {};
    let widget_name = &widget.name;
    for ((ident, key), value) in &widget.child_properties {
        let property_func = Ident::new(&format!("set_{}_{}", ident, key), key.span());
        tokens = quote_spanned! { widget_name.span() =>
            #tokens
            parent.#property_func(&self.widgets.#widget_name, #value);
        };
    }
    if !widget.child_properties.is_empty() {
        Some(block_to_impl_item(quote_spanned! { widget_name.span() =>
            fn on_add<W: ::relm::IsA<::gtk::Widget> + ::relm::IsA<::relm::Object>>(&self, parent: W) {
                let parent: gtk::Box = ::relm::Cast::downcast(::relm::Cast::upcast::<::gtk::Widget>(parent))
                    .expect("the parent of a widget with child properties must be a gtk::Box");
                #tokens
            }
        }))
    }
    else {
        None
    }
}

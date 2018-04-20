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

use std::collections::HashMap;

use proc_macro2::Span;
use quote::Tokens;
use syn::{
    Expr,
    Generics,
    Ident,
    Path,
    Type,
    TypePath,
    parse,
};
use syn::fold::Fold;
use syn::spanned::Spanned;

use parser::{
    Event,
    GtkWidget,
    RelmWidget,
    Widget,
};
use parser::EventValue::{CurrentWidget, ForeignWidget, NoEventValue};
use parser::EventValueReturn::{CallReturn, Return, WithoutReturn};
use parser::EitherWidget::{Gtk, Relm};
use transformer::Transformer;
use super::{Driver, MODEL_IDENT};

use self::WidgetType::*;
use self::WithParentheses::{WithParens, WithoutParens};

macro_rules! set_container {
    ($_self:expr, $widget:expr, $widget_name:expr, $widget_type:expr) => {
        if let Some(ref container_type) = $widget.container_type {
            if $_self.container_names.contains_key(container_type) {
                let attribute =
                    if let Some(ref typ) = *container_type {
                        format!("#[container=\"{}\"]", typ)
                    }
                    else {
                        "#[container]".to_string()
                    };
                panic!("Cannot use the {} attribute twice in the same widget", attribute);
            }
            $_self.relm_widgets.insert($widget_name.clone(), $widget_type.clone());
            $_self.container_names.insert(container_type.clone(), ($widget_name.clone(), $widget_type.clone()));
        }
    };
}

#[derive(PartialEq)]
enum WithParentheses {
    WithParens,
    WithoutParens,
}

#[derive(Clone, Copy, PartialEq)]
enum WidgetType {
    IsGtk,
    IsRelm,
}

pub fn gen(name: &Ident, widget: &Widget, driver: &mut Driver) -> (Tokens, HashMap<Ident, Path>, Tokens) {
    let mut generator = Generator::new(driver);
    let widget_tokens = generator.widget(widget, None, IsGtk);
    let driver = generator.driver.take().expect("driver");
    let idents: Vec<_> = driver.widgets.keys().collect();
    let root_widget_name = &driver.root_widget.as_ref().expect("root_widget is None");
    let widget_names1: Vec<_> = generator.widget_names.iter()
        .filter(|ident| (idents.contains(ident) || generator.relm_widgets.contains_key(ident))
                && ident != root_widget_name)
        .collect();
    let widget_names1 = &widget_names1;
    let widget_names2 = widget_names1;
    let events = &generator.events;
    let properties = &generator.properties;
    let model_ident = Ident::from(MODEL_IDENT);
    let code = quote_spanned! { name.span() =>
        #widget_tokens

        #(#events)*
        #(#properties)*

        #name {
            #root_widget_name: #root_widget_name,
            #(#widget_names1: #widget_names2,)*
            model: #model_ident,
        }
    };
    let container_impl = gen_container_impl(&generator, widget, driver.generic_types.as_ref().expect("generic types"));
    (code, generator.relm_widgets, container_impl)
}

struct Generator<'a> {
    container_names: HashMap<Option<String>, (Ident, Path)>,
    driver: Option<&'a mut Driver>,
    events: Vec<Tokens>,
    properties: Vec<Tokens>,
    relm_widgets: HashMap<Ident, Path>,
    widget_names: Vec<Ident>,
}

impl<'a> Generator<'a> {
    fn new(driver: &'a mut Driver) -> Self {
        Generator {
            container_names: HashMap::new(),
            driver: Some(driver),
            events: vec![],
            properties: vec![],
            relm_widgets: HashMap::new(),
            widget_names: vec![],
        }
    }

    fn add_child_or_show_all(&mut self, widget: &Widget, parent: Option<&Ident>, parent_widget_type: WidgetType)
        -> Tokens
    {
        let widget_name = &widget.name;
        if let Some(name) = parent {
            if parent_widget_type == IsGtk {
                quote_spanned! { widget_name.span() =>
                    ::gtk::ContainerExt::add(&#name, &#widget_name);
                }
            }
            else {
                quote! {
                    #name.add(&#widget_name);
                }
            }
        }
        else {
            let struct_name = &widget.typ;
            let driver = self.driver.as_mut().expect("driver");
            driver.root_widget_type = Some(quote! {
                #struct_name
            });
            driver.root_widget = Some(widget_name.clone());
            driver.root_widget_expr = Some(quote! {
                #widget_name
            });
            quote! {
            }
        }
    }

    fn add_or_create_widget(&mut self, parent: Option<&Ident>, parent_widget_type: WidgetType, widget_name: &Ident,
        widget_type_ident: &Path, init_parameters: &[Expr], is_container: bool) -> Tokens
    {
        let init_parameters = gen_model_param(init_parameters, WithParens);
        if let Some(parent) = parent {
            if parent_widget_type == IsGtk {
                let add_method =
                    if is_container {
                        quote! { add_container }
                    }
                    else {
                        quote! { add_widget }
                    };
                quote_spanned! { widget_name.span() =>
                    let #widget_name = {
                        ::relm::ContainerWidget::#add_method::<#widget_type_ident>(&#parent, #init_parameters)
                    };
                }
            }
            else {
                quote_spanned! { widget_name.span() =>
                    let #widget_name = {
                        #parent.add_widget::<#widget_type_ident>(#init_parameters)
                    };
                }
            }
        }
        else {
            let driver = self.driver.as_mut().expect("driver");
            driver.root_widget_type = Some(quote_spanned! { widget_name.span() =>
                <#widget_type_ident as ::relm::Widget>::Root
            });
            driver.root_widget = Some(widget_name.clone());
            driver.root_widget_expr = Some(quote! {
                #widget_name.widget()
            });
            if is_container {
                quote_spanned! { widget_name.span() =>
                    let #widget_name = ::relm::create_container::<#widget_type_ident>(#init_parameters);
                }
            }
            else {
                quote_spanned! { widget_name.span() =>
                    let #widget_name = ::relm::create_component::<#widget_type_ident>(#init_parameters);
                }
            }
        }
    }

    fn collect_event(&mut self, widget_name: Tokens, name: &Ident, event: &Event) {
        let event_ident = Ident::new(&format!("connect_{}", name), name.span());
        let event_params = &event.params;
        let shared_values = gen_shared_values(&event.shared_values);
        let metadata = gen_event_metadata(event);
        let connect =
            match event.value {
                CurrentWidget(WithoutReturn(ref event_value)) => quote_spanned! { widget_name.span() => {
                    #shared_values
                    connect!(relm, #widget_name, #event_ident(#(#event_params),*), #metadata #event_value);
                }},
                ForeignWidget(ref foreign_widget_name, WithoutReturn(ref event_value)) => quote! {{
                    #shared_values
                    connect!(#widget_name, #event_ident(#(#event_params),*), #foreign_widget_name, #event_value);
                }},
                CurrentWidget(Return(ref event_value, ref return_value)) => quote_spanned! { widget_name.span() => {
                    #shared_values
                    connect!(relm, #widget_name, #event_ident(#(#event_params),*), return (#event_value, #return_value));
                }},
                ForeignWidget(_, Return(_, _)) | ForeignWidget(_, CallReturn(_)) => unreachable!(),
                CurrentWidget(CallReturn(ref func)) => quote_spanned! { widget_name.span() => {
                    #shared_values
                    connect!(relm, #widget_name, #event_ident(#(#event_params),*), #metadata #func);
                }},
                NoEventValue => panic!("no event value"),
            };
        self.events.push(connect);
    }

    fn collect_events(&mut self, widget: &Widget, gtk_widget: &GtkWidget) {
        let widget_name = &widget.name;
        for (name, event) in &gtk_widget.events {
            self.collect_event(quote! { #widget_name }, name, event);
        }
        for (&(ref child_name, ref name), event) in &widget.child_events {
            let child_ident = Ident::new(&format!("get_{}", child_name), child_name.span());
            self.collect_event(quote! { #widget_name.#child_ident() }, name, event);
        }
    }

    fn collect_relm_events(&mut self, widget: &Widget, relm_widget: &RelmWidget) {
        let widget_name = &widget.name;
        for (event_ident, widget_events) in &relm_widget.events {
            for event in widget_events {
                let params =
                    if event.params.is_empty() {
                        quote! {}
                    }
                    else {
                        let event_params = &event.params;
                        quote! {
                            (#(#event_params),*)
                        }
                    };
                let metadata = gen_event_metadata(event);
                let connect =
                    match event.value {
                        CurrentWidget(WithoutReturn(ref event_value)) => quote_spanned! { widget_name.span() => {
                            connect!(#widget_name@#event_ident #params, relm, #metadata #event_value);
                        }},
                        ForeignWidget(ref foreign_widget_name, WithoutReturn(ref event_value)) => quote! {{
                            connect!(#widget_name@#event_ident #params, #foreign_widget_name,
                                     #metadata #event_value);
                        }},
                        CurrentWidget(Return(_, _)) | CurrentWidget(CallReturn(_)) | ForeignWidget(_, Return(_, _)) |
                            ForeignWidget(_, CallReturn(_)) => unreachable!(),
                        NoEventValue => panic!("no event value"),
                    };
                self.events.push(connect);
            }
        }
        for (name, event) in &relm_widget.gtk_events {
            self.collect_event(quote! { #widget_name.widget() }, name, event);
        }
        for (&(ref child_name, ref name), event) in &widget.child_events {
            let child_ident = Ident::new(&format!("get_{}", child_name), child_name.span());
            self.collect_event(quote! { #widget_name.widget().#child_ident() }, &name, event);
        }
    }

    fn gtk_set_prop_calls(&self, widget: &Widget, ident: Tokens) -> (Vec<Tokens>, Vec<Tokens>) {
        let mut properties = vec![];
        let mut visible_properties = vec![];
        for (key, value) in &widget.properties {
            let mut remover = Transformer::new(MODEL_IDENT);
            let new_value = remover.fold_expr(value.clone());
            let property_func = Ident::new(&format!("set_{}", key), key.span());
            let property = quote! {
                #ident.#property_func(#new_value);
            };
            if key == "visible" {
                visible_properties.push(property);
            }
            else {
                properties.push(property);
            }
        }
        (properties, visible_properties)
    }

    fn gtk_widget(&mut self, widget: &Widget, gtk_widget: &GtkWidget, parent: Option<&Ident>,
        parent_widget_type: WidgetType) -> Tokens
    {
        let struct_name = &widget.typ;
        let widget_name = &widget.name;
        set_container!(self, widget, widget_name, struct_name);
        self.widget_names.push(widget_name.clone());

        if gtk_widget.save {
            self.relm_widgets.insert(widget_name.clone(), struct_name.clone());
        }

        let construct_widget = gen_construct_widget(widget, gtk_widget);
        self.collect_events(widget, gtk_widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name), IsGtk))
            .collect();

        let add_child_or_show_all = self.add_child_or_show_all(widget, parent, parent_widget_type);
        let ident = quote! { #widget_name };
        let (properties, visible_properties) = self.gtk_set_prop_calls(widget, ident);
        let child_properties = gen_set_child_prop_calls(widget, parent, parent_widget_type, IsGtk);

        quote_spanned! { widget_name.span() =>
            let #widget_name: #struct_name = #construct_widget;
            #(#properties)*
            #(#children)*
            #add_child_or_show_all
            ::gtk::WidgetExt::show(&#widget_name);
            #(#visible_properties)*
            #(#child_properties)*
        }
    }

    fn relm_widget(&mut self, widget: &Widget, relm_widget: &RelmWidget, parent: Option<&Ident>,
        parent_widget_type: WidgetType) -> Tokens
    {
        self.widget_names.push(widget.name.clone());
        let widget_name = &widget.name;
        let widget_type_ident = &widget.typ;
        set_container!(self, widget, widget_name, widget_type_ident);
        let relm_component_type = gen_relm_component_type(widget.is_container, widget_type_ident);
        self.relm_widgets.insert(widget.name.clone(), relm_component_type);

        self.collect_relm_events(widget, relm_widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name), IsRelm))
            .collect();
        let ident = quote! { #widget_name.widget() };
        let (mut properties, mut visible_properties) = self.gtk_set_prop_calls(widget, ident);
        self.properties.append(&mut properties);
        self.properties.append(&mut visible_properties);

        let add_or_create_widget = self.add_or_create_widget(
            parent, parent_widget_type, widget_name, widget_type_ident, &widget.init_parameters, widget.is_container);
        let child_properties = gen_set_child_prop_calls(widget, parent, parent_widget_type, IsRelm);
        let messages = self.messages(widget, relm_widget);

        quote! {
            #add_or_create_widget
            #messages
            #(#children)*
            #(#child_properties)*
        }
    }

    fn messages(&self, widget: &Widget, relm_widget: &RelmWidget) -> Tokens {
        let mut tokens = quote! {};
        let name = &widget.name;
        for (variant, value) in &relm_widget.messages {
            let mut remover = Transformer::new(MODEL_IDENT);
            let value = remover.fold_expr(value.clone());
            tokens = quote! {
                #tokens
                #name.stream().emit(#variant(#value));
            };
        }
        tokens
    }

    fn widget(&mut self, widget: &Widget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Tokens {
        match widget.widget {
            Gtk(ref gtk_widget) => self.gtk_widget(widget, gtk_widget, parent, parent_widget_type),
            Relm(ref relm_widget) => self.relm_widget(widget, relm_widget, parent, parent_widget_type),
        }
    }
}

fn gen_construct_widget(widget: &Widget, gtk_widget: &GtkWidget) -> Tokens {
    let struct_name = &widget.typ;

    let properties_count = gtk_widget.construct_properties.len() as u32;
    let mut values = vec![];
    let mut parameters = vec![];
    for (key, value) in gtk_widget.construct_properties.iter() {
        let mut remover = Transformer::new(MODEL_IDENT);
        let value = remover.fold_expr(value.clone());
        let key = key.to_string();
        values.push(quote_spanned! { struct_name.span() =>
            ::gtk::ToValue::to_value(&#value)
        });
        let index = parameters.len();
        parameters.push(quote_spanned! { struct_name.span() =>
            ::relm::GParameter {
                name: ::relm::ToGlibPtr::to_glib_full(#key),
                value: ::std::ptr::read(::relm::ToGlibPtr::to_glib_none(&values[#index]).0),
            }
        });
    }
    // TODO: use this new code when g_object_new_with_properties() is released.
    /*let mut names = vec![];
    let mut values = vec![];
    for (key, value) in gtk_widget.construct_properties.iter() {
        let key = key.to_string();
        names.push(quote! {
            #key
        });
        values.push(quote! {
            &#value
        });
    }*/

    if widget.init_parameters.is_empty() {
        quote_spanned! { struct_name.span() =>
            unsafe {
                use gtk::StaticType;
                use relm::{Downcast, FromGlibPtrNone};
                let values: &[::gtk::Value] = &[#(#values),*];
                let mut parameters = [#(#parameters),*];
                ::gtk::Widget::from_glib_none(::relm::g_object_newv(
                    ::relm::ToGlib::to_glib(&#struct_name::static_type()),
                    #properties_count, parameters.as_mut_ptr()) as *mut _)
                    .downcast_unchecked()
                // TODO: use this new code when g_object_new_with_properties() is released.
                /*let names: &[&str] = &[#(#names),*];
                let values: &[&::gtk::ToValue] = &[#(#values),*];
                ::gtk::Widget::from_glib_none(::relm::g_object_new_with_properties(#struct_name::static_type().to_glib(),
                    #properties_count, ::relm::ToGlibPtr::to_glib_full(&names),
                    ::relm::ToGlibPtr::to_glib_full(&values) as *mut _) as *mut _)
                .downcast_unchecked()*/
            }
        }
    }
    else {
        let params = gen_model_param(&widget.init_parameters, WithoutParens);
        quote! {
            #struct_name::new(#params)
        }
    }
}

fn gen_event_metadata(event: &Event) -> Tokens {
    if let CurrentWidget(CallReturn(_)) = event.value {
        quote! {
            return
        }
    }
    else {
        quote! {}
    }
}

fn gen_shared_values(shared_values: &[Ident]) -> Tokens {
    let model_ident = Ident::from(MODEL_IDENT);
    let fields = shared_values.iter()
        .map(|ident| {
             let typ =
                 if ident == "relm" {
                     quote_spanned! { ident.span() =>
                         Relm<_>
                     }
                 }
                 else {
                     quote_spanned! { ident.span() =>
                         ::std::rc::Rc<_>
                     }
                 };
             quote! {
                 let #ident: #typ = #model_ident.#ident.clone();
             }
        });
    quote! {
        #(#fields)*
    }
}

fn gen_widget_ident(widget: &Widget) -> (Tokens, Span) {
    match widget.widget {
        Gtk(ref gtk_widget) => {
            if let Some(Type::Path(TypePath { path: Path { ref segments, .. }, .. })) = gtk_widget.relm_name {
                let ident = &segments[0].ident;
                (quote! {
                    #ident
                }, ident.span())
            }
            else {
                panic!("relm name should be a Path");
            }
        },
        Relm(_) => {
            let path = &widget.typ;
            (quote! {
                #path
            }, path.span())
        },
    }
}

fn gen_widget_type(widget: &Widget) -> Tokens {
    match widget.widget {
        Gtk(ref gtk_widget) => {
            let ident = gtk_widget.relm_name.as_ref().unwrap();
            quote! {
                #ident
            }
        },
        Relm(_) => {
            let path = &widget.typ;
            quote! {
                #path
            }
        },
    }
}

fn gen_add_widget_method(container_names: &HashMap<Option<String>, (Ident, Path)>) -> Tokens {
    if container_names.len() > 1 {
        let span = container_names.values().next().expect("at least one container name").0.span();
        let mut default_container = Tokens::new();
        let mut other_containers = Tokens::new();
        for (parent_id, &(ref name, _)) in container_names {
            if parent_id.is_none() {
                default_container = quote_spanned! { span =>
                    ::gtk::ContainerExt::add(&container.container, widget.widget());
                    ::gtk::Cast::upcast(container.container.clone())
                };
            }
            else {
                if other_containers == quote! {} {
                    other_containers = quote_spanned! { span =>
                        if WIDGET::parent_id() == Some(#parent_id) {
                            ::gtk::ContainerExt::add(&container.containers.#name, widget.widget());
                            ::gtk::Cast::upcast(container.containers.#name.clone())
                        }
                    };
                }
                else {
                    other_containers = quote_spanned! { span =>
                        #other_containers
                        else if WIDGET::parent_id() == Some(#parent_id) {
                            ::gtk::ContainerExt::add(&container.containers.#name, widget.widget());
                            ::gtk::Cast::upcast(container.containers.#name.clone())
                        }
                    };
                }
            }
        }
        if other_containers != quote! {} {
            default_container = quote! {
                else {
                    #default_container
                }
            };
        }
        quote_spanned! { span =>
            fn add_widget<WIDGET: Widget>(container: &::relm::ContainerComponent<Self>,
                widget: &::relm::Component<WIDGET>) -> ::gtk::Container
            {
                #other_containers
                #default_container
            }
        }
    }
    else {
        quote! {
        }
    }
}

fn gen_container_impl(generator: &Generator, widget: &Widget, generic_types: &Generics) -> Tokens {
    let where_clause = gen_where_clause(generic_types);
    let widget_type = gen_widget_type(widget);
    if generator.container_names.is_empty() {
        quote! {
        }
    }
    else if !generator.container_names.contains_key(&None) {
        panic!("Use of #[container=\"name\"] attribute without the default #[container].");
    }
    else {
        let mut container_type = None;
        for (ident, &(_, ref typ)) in &generator.container_names {
            if ident.is_none() {
                container_type = Some(typ);
            }
        }
        let typ = container_type.expect("container type");
        let &(ref name, _) = generator.container_names.get(&None).expect("default container");
        let add_widget_method = gen_add_widget_method(&generator.container_names);

        let (widget_ident, widget_ident_span) = gen_widget_ident(widget);
        let (containers, containers_type, other_containers_func) = gen_other_containers(&generator, &widget_ident,
            widget_ident_span);

        quote_spanned! { widget.name.span() =>
            #containers

            impl #generic_types ::relm::Container for #widget_type #where_clause {
                type Container = #typ;
                type Containers = #containers_type;

                fn container(&self) -> &Self::Container {
                    &self.#name
                }

                #other_containers_func

                #add_widget_method
            }
        }
    }
}

fn gen_other_containers(generator: &Generator, widget_type: &Tokens, widget_ident_span: Span) ->
    (Tokens, Tokens, Tokens)
{
    if generator.container_names.len() > 1 {
        let containers_ident = Ident::new(&format!("{}Containers", widget_type), widget_ident_span);
        let mut names = vec![];
        let mut types = vec![];
        let mut values = vec![];
        let mut suffixes = vec![];
        for (_, &(ref name, ref typ)) in &generator.container_names {
            names.push(name.clone());
            let mut original_type = typ.clone();
            let typ =
                if typ.segments.len() > 1 {
                    // GTK+ widget
                    values.push(name.clone());
                    suffixes.push(quote! {});
                    quote! { #original_type }
                }
                else {
                    // Relm widget
                    values.push(Ident::new(&format!("{}", name), name.span()));
                    suffixes.push(quote! { .widget() });
                    let original_ident = original_type.segments[0].ident.clone();
                    quote_spanned! { original_ident.span() =>
                        <#original_ident as ::relm::Widget>::Root
                    }
                };
            types.push(typ);
        }
        let names = &names;
        (quote! {
            #[allow(dead_code)]
            #[derive(Clone)]
            pub struct #containers_ident {
                #(#names: #types,)*
            }
        }, quote! {
            #containers_ident
        }, quote! {
            fn other_containers(&self) -> Self::Containers {
                #containers_ident {
                    #(#names: self.#values#suffixes.clone(),)*
                }
            }
        })
    }
    else {
        (quote! {
        }, quote! {
            ()
        }, quote! {
            fn other_containers(&self) -> Self::Containers {
            }
        })
    }
}

fn gen_model_param(init_parameters: &[Expr], with_parens: WithParentheses) -> Tokens {
    let mut params = vec![];
    for param in init_parameters {
        let mut remover = Transformer::new(MODEL_IDENT);
        let value = remover.fold_expr(param.clone());
        params.push(value);
    }
    if with_parens == WithParens && (params.len() > 1 || params.is_empty()) {
        quote! {
            (#(#params),*)
        }
    }
    else {
        quote! {
            #(#params),*
        }
    }
}

fn gen_relm_component_type(is_container: bool, name: &Path) -> Path {
    let tokens =
        if is_container {
            quote_spanned! { name.span() =>
                ::relm::ContainerComponent<#name>
            }
        }
        else {
            quote_spanned! { name.span() =>
                ::relm::Component<#name>
            }
        };
    parse(tokens.into()).expect("gen_relm_component_type is a Path")
}

fn gen_set_child_prop_calls(widget: &Widget, parent: Option<&Ident>, parent_widget_type: WidgetType,
    widget_type: WidgetType) -> Vec<Tokens>
{
    let widget_name = &widget.name;
    let mut child_properties = vec![];
    if let Some(parent) = parent {
        for (&(ref ident, ref key), value) in &widget.child_properties {
            let property_func = Ident::new(&format!("set_{}_{}", ident, key), key.span());
            let parent =
                if parent_widget_type == IsGtk {
                    quote! {
                        #parent
                    }
                }
                else {
                    quote! {
                        #parent.container
                    }
                };
            let call =
                if widget_type == IsGtk {
                    quote! {
                        #parent.#property_func(&#widget_name, #value);
                    }
                }
                else {
                    quote! {
                        #parent.#property_func(#widget_name.widget(), #value);
                    }
                };
            child_properties.push(call);
        }
    }
    child_properties
}

pub fn gen_where_clause(generics: &Generics) -> Tokens {
    let where_clause = &generics.where_clause;
    // TODO: check that it is okay (vs what we did before).
    quote! {
        #where_clause
    }
    /*if generics.where_clause.predicates.is_empty() {
        quote! {
        }
    }
    else {
        let where_clause = &generics.where_clause;
        quote! {
            #where_clause
        }
    }*/
}

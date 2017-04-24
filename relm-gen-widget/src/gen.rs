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

use quote::Tokens;
use syn::Ident;

use parser::{GtkWidget, RelmWidget, Widget};
use parser::EventValue::{CurrentWidget, ForeignWidget};
use parser::EventValueReturn::{CallReturn, Return, WithoutReturn};
use parser::Widget::{Gtk, Relm};

use self::WidgetType::*;

macro_rules! gen_set_prop_calls {
    ($widget:expr, $ident:expr) => {{
        let ident = $ident;
        let mut properties = vec![];
        let mut visible_properties = vec![];
        for (key, value) in &$widget.properties {
            let property_func = Ident::new(format!("set_{}", key));
            let property = quote! {
                #ident.#property_func(#value);
            };
            if key == "visible" {
                visible_properties.push(property);
            }
            else {
                properties.push(property);
            }
        }
        (properties, visible_properties)
    }};
}

macro_rules! set_container {
    ($_self:expr, $widget:expr, $widget_name:expr, $widget_type:expr) => {
        if $widget.is_container {
            if $_self.container_name.is_some() {
                panic!("Cannot use the #[container] attribute twice in the same widget");
            }
            $_self.relm_widgets.insert($widget_name.clone(), $widget_type.clone());
            $_self.container_name = Some($widget_name.clone());
            $_self.container_type = Some($widget_type.clone());
        }
    };
}

#[derive(Clone, Copy, PartialEq)]
enum WidgetType {
    IsGtk,
    IsRelm,
}

pub fn gen(name: &Ident, widget: &Widget, root_widget: &mut Option<Ident>, root_widget_expr: &mut Option<Tokens>, root_widget_type: &mut Option<Ident>, idents: &[&Ident]) -> (Tokens, HashMap<Ident, Ident>, Tokens) {
    let mut generator = Generator::new(root_widget, root_widget_expr, root_widget_type);
    let widget_tokens = generator.widget(widget, None, IsGtk);
    let root_widget_name = &generator.root_widget.as_ref().expect("root_widget is None");
    let widget_names1: Vec<_> = generator.widget_names.iter()
        .filter(|ident| (idents.contains(ident) || generator.relm_widgets.contains_key(ident)) && ident != root_widget_name)
        .collect();
    let widget_names1 = &widget_names1;
    let widget_names2 = widget_names1;
    let events = &generator.events;
    let code = quote! {
        #widget_tokens

        #(#events)*

        #name {
            #root_widget_name: #root_widget_name,
            #(#widget_names1: #widget_names2),*
        }
    };
    let container_impl = gen_container_impl(&generator, widget);
    (code, generator.relm_widgets, container_impl)
}

struct Generator<'a> {
    container_name: Option<Ident>,
    container_type: Option<Ident>,
    events: Vec<Tokens>,
    relm_widgets: HashMap<Ident, Ident>,
    root_widget: &'a mut Option<Ident>,
    root_widget_expr: &'a mut Option<Tokens>,
    root_widget_type: &'a mut Option<Ident>,
    widget_names: Vec<Ident>,
}

impl<'a> Generator<'a> {
    fn new(root_widget: &'a mut Option<Ident>, root_widget_expr: &'a mut Option<Tokens>, root_widget_type: &'a mut Option<Ident>) -> Self {
        Generator {
            container_name: None,
            container_type: None,
            events: vec![],
            relm_widgets: HashMap::new(),
            root_widget: root_widget,
            root_widget_expr: root_widget_expr,
            root_widget_type: root_widget_type,
            widget_names: vec![],
        }
    }

    fn add_child_or_show_all(&mut self, widget: &GtkWidget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Tokens {
        let widget_name = &widget.name;
        if let Some(name) = parent {
            if parent_widget_type == IsGtk {
                quote! {
                    ::gtk::ContainerExt::add(&#name, &#widget_name);
                }
            }
            else {
                quote! {
                    ::relm::RelmContainer::add(&#name, &#widget_name);
                }
            }
        }
        else {
            let struct_name = &widget.gtk_type;
            *self.root_widget_type = Some(struct_name.clone());
            *self.root_widget = Some(widget_name.clone());
            *self.root_widget_expr = Some(quote! {
                #widget_name
            });
            quote! {
            }
        }
    }

    fn add_or_create_widget(&mut self, parent: Option<&Ident>, parent_widget_type: WidgetType, widget_name: &Ident, widget_type_ident: Ident) -> Tokens {
        if let Some(parent) = parent {
            if parent_widget_type == IsGtk {
                quote! {
                    let #widget_name = {
                        ::relm::ContainerWidget::add_widget::<#widget_type_ident, _>(&#parent, &relm)
                    };
                }
            }
            else {
                quote! {
                    let #widget_name = {
                        ::relm::RelmContainer::add_widget::<#widget_type_ident, _>(&#parent, &relm)
                    };
                }
            }
        }
        else {
            *self.root_widget_type = Some(Ident::new(format!("<{} as ::relm::Widget>::Root", widget_type_ident)));
            *self.root_widget = Some(widget_name.clone());
            *self.root_widget_expr = Some(quote! {
                #widget_name.widget().root()
            });
            quote! {
                let #widget_name = {
                    ::relm::create_component::<#widget_type_ident, _>(&relm)
                };
            }
        }
    }

    fn collect_events(&mut self, widget: &GtkWidget) {
        let widget_name = &widget.name;
        for (name, event) in &widget.events {
            let event_ident = Ident::new(format!("connect_{}", name));
            let event_params: Vec<_> = event.params.iter().map(|ident| Ident::new(ident.as_ref())).collect();
            let connect =
                match event.value {
                    CurrentWidget(WithoutReturn(ref event_value)) => quote! {
                        connect!(relm, #widget_name, #event_ident(#(#event_params),*), #event_value);
                    },
                    ForeignWidget(ref foreign_widget_name, WithoutReturn(ref event_value)) => quote! {
                        connect!(#widget_name, #event_ident(#(#event_params),*), #foreign_widget_name, #event_value);
                    },
                    CurrentWidget(Return(ref event_value, ref return_value)) => quote! {
                        connect!(relm, #widget_name, #event_ident(#(#event_params),*) (#event_value, #return_value));
                    },
                    ForeignWidget(_, Return(_, _)) | ForeignWidget(_, CallReturn(_)) => unreachable!(),
                    CurrentWidget(CallReturn(ref func)) => quote! {
                        connect!(relm, #widget_name, #event_ident(#(#event_params),*) #func);
                    },

                };
            self.events.push(connect);
        }
    }

    fn collect_relm_events(&mut self, widget: &RelmWidget) {
        let widget_name = &widget.name;
        for (name, widget_events) in &widget.events {
            let event_ident = Ident::new(name.as_ref());
            for event in widget_events {
                let params =
                    if event.params.is_empty() {
                        quote! {}
                    }
                    else {
                        let event_params: Vec<_> = event.params.iter().map(|ident| Ident::new(ident.as_ref())).collect();
                        quote! {
                            (#(#event_params),*)
                        }
                    };
                let connect =
                    match event.value {
                        CurrentWidget(WithoutReturn(ref event_value)) => quote! {
                            connect!(#widget_name@#event_ident #params, relm, #event_value);
                        },
                        ForeignWidget(ref foreign_widget_name, WithoutReturn(ref event_value)) => quote! {
                            connect!(#widget_name@#event_ident #params, #foreign_widget_name, #event_value);
                        },
                        CurrentWidget(Return(_, _)) | CurrentWidget(CallReturn(_)) | ForeignWidget(_, Return(_, _)) | ForeignWidget(_, CallReturn(_)) => unreachable!(),
                    };
                self.events.push(connect);
            }
        }
    }

    fn gtk_widget(&mut self, widget: &GtkWidget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Tokens {
        let struct_name = &widget.gtk_type;
        let widget_name = &widget.name;
        set_container!(self, widget, widget_name, struct_name);
        self.widget_names.push(widget_name.clone());

        if widget.save {
            self.relm_widgets.insert(widget_name.clone(), struct_name.clone());
        }

        let construct_widget = gen_construct_widget(widget);
        self.collect_events(widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name), IsGtk))
            .collect();

        let add_child_or_show_all = self.add_child_or_show_all(widget, parent, parent_widget_type);
        let ident = quote! { #widget_name };
        let (properties, visible_properties) = gen_set_prop_calls!(widget, ident);
        let child_properties = gen_set_child_prop_calls(widget, parent, parent_widget_type);

        quote! {
            let #widget_name: #struct_name = #construct_widget;
            #(#properties)*
            #(#children)*
            #add_child_or_show_all
            #widget_name.show();
            #(#visible_properties)*
            #(#child_properties)*
        }
    }

    fn relm_widget(&mut self, widget: &RelmWidget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Tokens {
        self.widget_names.push(widget.name.clone());
        let widget_name = &widget.name;
        let widget_type_ident = Ident::new(widget.relm_type.as_ref());
        set_container!(self, widget, widget_name, widget_type_ident);
        let relm_component_type = gen_relm_component_type(&widget.relm_type);
        self.relm_widgets.insert(widget.name.clone(), relm_component_type);

        self.collect_relm_events(widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name), IsRelm))
            .collect();
        let ident = quote! { #widget_name.widget() };
        let (properties, visible_properties) = gen_set_prop_calls!(widget, ident);

        let add_or_create_widget = self.add_or_create_widget(parent, parent_widget_type, widget_name, widget_type_ident);

        quote! {
            #add_or_create_widget
            #(#properties)*
            #(#visible_properties)*
            #(#children)*
        }
    }

    fn widget(&mut self, widget: &Widget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Tokens {
        match *widget {
            Gtk(ref gtk_widget) => self.gtk_widget(gtk_widget, parent, parent_widget_type),
            Relm(ref relm_widget) => self.relm_widget(relm_widget, parent, parent_widget_type),
        }
    }
}

fn gen_construct_widget(widget: &GtkWidget) -> Tokens {
    let struct_name = &widget.gtk_type;

    let mut params = Tokens::new();
    for param in &widget.init_parameters {
        params.append(param);
        params.append(",");
    }

    if widget.init_parameters.is_empty() {
        quote! {
            unsafe {
                use gtk::StaticType;
                use relm::{Downcast, FromGlibPtrNone, ToGlib};
                ::gtk::Widget::from_glib_none(::relm::g_object_new(#struct_name::static_type().to_glib(),
                #params ::std::ptr::null() as *const i8) as *mut _)
                .downcast_unchecked()
            }
        }
    }
    else {
        quote! {
            #struct_name::new(#params)
        }
    }
}

fn gen_container_impl(generator: &Generator, widget: &Widget) -> Tokens {
    let widget_type =
        match *widget {
            Gtk(ref gtk_widget) => gtk_widget.relm_name.as_ref().unwrap(),
            Relm(ref relm_widget) => &relm_widget.relm_type,
        };
    match (&generator.container_name, &generator.container_type) {
        (&Some(ref name), &Some(ref typ)) => {
            quote! {
                impl ::relm::Container for #widget_type {
                    type Container = #typ;

                    fn container(&self) -> &Self::Container {
                        &self.#name
                    }
                }
            }
        },
        _ => quote! {},
    }
}

fn gen_relm_component_type(name: &Ident) -> Ident {
    Ident::new(format!("::relm::Component<{0}>", name).as_ref())
}

fn gen_set_child_prop_calls(widget: &GtkWidget, parent: Option<&Ident>, parent_widget_type: WidgetType) -> Vec<Tokens> {
    let widget_name = &widget.name;
    let mut child_properties = vec![];
    if let Some(parent) = parent {
        for (key, value) in &widget.child_properties {
            let property_func = Ident::new(format!("set_child_{}", key));
            let parent =
                if parent_widget_type == IsGtk {
                    quote! {
                        #parent
                    }
                }
                else {
                    quote! {
                        ::relm::Container::container(#parent.widget())
                    }
                };
            child_properties.push(quote! {
                #parent.#property_func(&#widget_name, #value);
            });
        }
    }
    child_properties
}

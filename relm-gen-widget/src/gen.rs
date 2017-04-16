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
use parser::EventValueReturn::{Return, WithoutReturn};
use parser::Widget::{Gtk, Relm};

pub fn gen(name: &Ident, widget: &Widget, root_widget: &mut Option<Ident>, root_widget_type: &mut Option<Ident>, idents: &[&Ident]) -> (Tokens, HashMap<Ident, Ident>) {
    let mut generator = Generator::new(root_widget, root_widget_type);
    let widget = generator.widget(widget, None);
    let widget_names1: Vec<_> = generator.widget_names.iter()
        .filter(|ident| idents.contains(ident) || generator.relm_widgets.contains_key(ident))
        .collect();
    let widget_names1 = &widget_names1;
    let widget_names2 = widget_names1;
    let root_widget_name = &generator.root_widget.as_ref().unwrap();
    let events = &generator.events;
    let code = quote! {
        #widget

        #(#events)*

        #name {
            #root_widget_name: #root_widget_name,
            #(#widget_names1: #widget_names2),*
        }
    };
    (code, generator.relm_widgets)
}

struct Generator<'a> {
    events: Vec<Tokens>,
    relm_widgets: HashMap<Ident, Ident>,
    root_widget: &'a mut Option<Ident>,
    root_widget_type: &'a mut Option<Ident>,
    widget_names: Vec<Ident>,
}

impl<'a> Generator<'a> {
    fn new(root_widget: &'a mut Option<Ident>, root_widget_type: &'a mut Option<Ident>) -> Self {
        Generator {
            events: vec![],
            relm_widgets: HashMap::new(),
            root_widget: root_widget,
            root_widget_type: root_widget_type,
            widget_names: vec![],
        }
    }

    fn add_child_or_show_all(&mut self, widget: &GtkWidget, parent: Option<&Ident>) -> Tokens {
        let widget_name = &widget.name;
        if let Some(name) = parent {
            quote! {
                ::gtk::ContainerExt::add(&#name, &#widget_name);
            }
        }
        else {
            let struct_name = &widget.gtk_type;
            *self.root_widget_type = Some(struct_name.clone());
            *self.root_widget = Some(widget_name.clone());
            quote! {
                #widget_name.show_all();
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
                    ForeignWidget(_, Return(_, _)) => unreachable!(),
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
                        CurrentWidget(Return(_, _)) | ForeignWidget(_, Return(_, _)) => unreachable!(),
                    };
                self.events.push(connect);
            }
        }
    }

    fn gtk_widget(&mut self, widget: &GtkWidget, parent: Option<&Ident>) -> Tokens {
        let struct_name = &widget.gtk_type;
        let widget_name = &widget.name;
        self.widget_names.push(widget_name.clone());

        if widget.save {
            self.relm_widgets.insert(widget_name.clone(), struct_name.clone());
        }

        let construct_widget = gen_construct_widget(widget);
        self.collect_events(widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name))).collect();

        let add_child_or_show_all = self.add_child_or_show_all(widget, parent);
        let properties = gen_set_prop_calls(widget);
        let child_properties = gen_set_child_prop_calls(widget, parent);

        quote! {
            let #widget_name: #struct_name = #construct_widget;
            #(#properties)*
            #(#children)*
            #add_child_or_show_all
            #(#child_properties)*
        }
    }

    fn relm_widget(&mut self, widget: &RelmWidget, parent: Option<&Ident>) -> Tokens {
        self.widget_names.push(widget.name.clone());
        let widget_name = &widget.name;
        let widget_type = Ident::new(widget.relm_type.as_ref());
        let relm_component_type = gen_relm_component_type(&widget.relm_type);
        self.relm_widgets.insert(widget.name.clone(), relm_component_type);
        let parent = parent.unwrap();

        self.collect_relm_events(widget);

        let children: Vec<_> = widget.children.iter()
            .map(|child| self.widget(child, Some(widget_name))).collect();

        quote! {
            let #widget_name = {
                ::relm::ContainerWidget::add_widget::<#widget_type, _>(&#parent, &relm)
            };
            #(#children)*
        }
    }

    fn widget(&mut self, widget: &Widget, parent: Option<&Ident>) -> Tokens {
        match *widget {
            Gtk(ref gtk_widget) => self.gtk_widget(gtk_widget, parent),
            Relm(ref relm_widget) => self.relm_widget(relm_widget, parent),
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

fn gen_relm_component_type(name: &Ident) -> Ident {
    Ident::new(format!("::relm::Component<<{0} as ::relm::Widget>::Model, <{0} as ::relm::Widget>::Msg, <{0} as ::relm::Widget>::Container>", name).as_ref())
}

fn gen_set_child_prop_calls(widget: &GtkWidget, parent: Option<&Ident>) -> Vec<Tokens> {
    let widget_name = &widget.name;
    let mut child_properties = vec![];
    for (key, value) in &widget.child_properties {
        let property_func = Ident::new(format!("set_child_{}", key));
        let parent = parent.expect("child properties only allowed for non-root widgets");
        child_properties.push(quote! {
            #parent.#property_func(&#widget_name, #value);
        });
    }
    child_properties
}

fn gen_set_prop_calls(widget: &GtkWidget) -> Vec<Tokens> {
    let widget_name = &widget.name;
    let mut properties = vec![];
    for (key, value) in &widget.properties {
        let property_func = Ident::new(format!("set_{}", key));
        properties.push(quote! {
            #widget_name.#property_func(#value);
        });
    }
    properties
}

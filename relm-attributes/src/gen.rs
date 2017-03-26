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
use parser::Widget::{Gtk, Relm};
use super::COMPONENTS;

pub fn gen(name: &Ident, widget: Widget, root_widget: &mut Option<Ident>, root_widget_type: &mut Option<Ident>, idents: Vec<&Ident>) -> (Tokens, HashMap<Ident, Ident>) {
    let mut widget_names = vec![];
    let mut relm_widgets = HashMap::new();
    let widget = gen_widget(&widget, None, &mut widget_names, root_widget, root_widget_type, &mut relm_widgets);
    let widget_names1: Vec<_> = widget_names.iter()
        .filter(|ident| idents.contains(ident) || relm_widgets.contains_key(ident))
        .collect();
    let widget_names1 = &widget_names1;
    let widget_names2 = widget_names1;
    let root_widget_name = &root_widget.as_ref().unwrap();
    let code = quote! {
        #widget

        #name {
            #root_widget_name: #root_widget_name,
            #(#widget_names1: #widget_names2),*
        }
    };
    (code, relm_widgets)
}

fn gen_widget(widget: &Widget, parent: Option<&Ident>, widget_names: &mut Vec<Ident>, root_widget: &mut Option<Ident>, root_widget_type: &mut Option<Ident>, relm_widgets: &mut HashMap<Ident, Ident>) -> Tokens {
    match *widget {
        Gtk(ref gtk_widget) => gen_gtk_widget(gtk_widget, parent, widget_names, root_widget, root_widget_type, relm_widgets),
        Relm(ref relm_widget) => gen_relm_widget(relm_widget, parent, widget_names, relm_widgets),
    }
}

fn gen_gtk_widget(widget: &GtkWidget, parent: Option<&Ident>, widget_names: &mut Vec<Ident>, root_widget: &mut Option<Ident>, root_widget_type: &mut Option<Ident>, relm_widgets: &mut HashMap<Ident, Ident>) -> Tokens {
    let struct_name = &widget.gtk_type;
    let widget_name = &widget.name;
    widget_names.push(widget_name.clone());

    let mut params = Tokens::new();
    for param in &widget.init_parameters {
        params.append(param);
        params.append(",");
    }

    let mut events = vec![];
    for (name, event) in &widget.events {
        let event_ident = Ident::new(format!("connect_{}", name));
        let event_params: Vec<_> = event.params.iter().map(|ident| Ident::new(ident.as_ref())).collect();
        let event_value = &event.value;
        events.push(quote! {
            connect!(relm, #widget_name, #event_ident(#(#event_params),*) #event_value);
        });
    }

    let children: Vec<_> = widget.children.iter()
        .map(|child| gen_widget(child, Some(widget_name), widget_names, root_widget, root_widget_type, relm_widgets)).collect();

    let add_child_or_show_all =
        if let Some(name) = parent {
            quote! {
                #name.add(&#widget_name);
            }
        }
        else {
            *root_widget_type = Some(struct_name.clone());
            *root_widget = Some(widget_name.clone());
            quote! {
                #widget_name.show_all();
            }
        };

    let mut properties = vec![];
    for (key, value) in &widget.properties {
        let property_func = Ident::new(format!("set_{}", key));
        properties.push(quote! {
            #widget_name.#property_func(#value);
        });
    }

    quote! {
        let #widget_name: #struct_name = unsafe {
            use gtk::StaticType;
            use relm::{Downcast, FromGlibPtrNone, ToGlib};
            ::gtk::Widget::from_glib_none(::relm::g_object_new(#struct_name::static_type().to_glib(),
                ::std::ptr::null()) as *mut _)
                .downcast_unchecked()
        };
        #(#properties)*
        #(#events)*
        #(#children)*
        #add_child_or_show_all
    }
}

fn gen_relm_widget(widget: &RelmWidget, parent: Option<&Ident>, widget_names: &mut Vec<Ident>, relm_widgets: &mut HashMap<Ident, Ident>) -> Tokens {
    widget_names.push(widget.name.clone());
    let widget_name = &widget.name;
    let widget_type = Ident::new(widget.relm_type.as_ref());
    let relm_component_type = gen_relm_component_type(&widget.relm_type);
    relm_widgets.insert(widget.name.clone(), relm_component_type);
    let parent = parent.unwrap();
    quote! {
        let #widget_name = {
            use ::relm::ContainerWidget;
            #parent.add_widget::<#widget_type, _, _>(&relm)
        };
    }
}

fn gen_relm_component_type(name: &Ident) -> Ident {
    let components = COMPONENTS.lock().unwrap();
    let model_type = &components[name].model_type;
    let msg_type = &components[name].msg_type;
    let view_type = &components[name].view_type;

    let mut model = Tokens::new();
    model.append_all(&[model_type]);

    let mut msg = Tokens::new();
    msg.append_all(&[msg_type]);

    let mut view = Tokens::new();
    view.append(view_type);

    Ident::new(format!("::relm::Component<{}, {}, {}>", model, msg, view).as_ref())
}

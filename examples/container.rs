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

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    ContainerExt,
    EventBox,
    Inhibit,
    Label,
    WidgetExt,
    Window,
};
use gtk::Orientation::Vertical;
use gtk::WindowType::Toplevel;
use relm::{Component, Container, ContainerWidget, RelmContainer, RemoteRelm, Widget};

use self::Msg::*;

#[derive(Clone)]
struct Button {
    button: gtk::Button,
}

impl Widget for Button {
    type Model = ();
    type Msg = ();
    type Root = gtk::Button;

    fn model() -> () {
    }

    fn root(&self) -> &Self::Root {
        &self.button
    }

    fn update(&mut self, _msg: (), _model: &mut ()) {
    }

    fn view(_relm: &RemoteRelm<Self>, _model: &()) -> Self {
        let button = gtk::Button::new_with_label("+");
        Button {
            button: button,
        }
    }
}

#[derive(Clone)]
struct VBox {
    event_box: EventBox,
    vbox: gtk::Box,
}

impl Container for VBox {
    type Container = gtk::Box;

    fn container(&self) -> &Self::Container {
        &self.vbox
    }
}

impl Widget for VBox {
    type Model = ();
    type Msg = ();
    type Root = EventBox;

    fn model() -> () {
        ()
    }

    fn root(&self) -> &Self::Root {
        &self.event_box
    }

    fn update(&mut self, _event: (), _model: &mut ()) {
    }

    fn view(_relm: &RemoteRelm<Self>, _model: &Self::Model) -> Self {
        let event_box = EventBox::new();
        let vbox = gtk::Box::new(Vertical, 0);
        event_box.add(&vbox);
        VBox {
            event_box: event_box,
            vbox: vbox,
        }
    }
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[derive(Clone)]
struct Win {
    button: Component<Button>,
    vbox: Component<VBox>,
    window: Window,
}

impl Widget for Win {
    type Model = ();
    type Msg = Msg;
    type Root = Window;

    fn model() -> () {
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &RemoteRelm<Self>, _model: &()) -> Self {
        let window = Window::new(Toplevel);
        let vbox = window.add_widget::<VBox, _>(&relm);
        let plus_button = gtk::Button::new_with_label("+");
        vbox.add(&plus_button);
        let label = Label::new(Some("0"));
        vbox.add(&label);
        let button = vbox.add_widget::<Button, _>(&relm);
        let minus_button = gtk::Button::new_with_label("-");
        vbox.add(&minus_button);
        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));
        window.show_all();
        Win {
            button: button,
            vbox: vbox,
            window: window,
        }
    }
}

fn main() {
    relm::run::<Win>().unwrap();
}

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
#[macro_use]
extern crate relm_derive;

use gtk::{
    Cast,
    ContainerExt,
    Inhibit,
    IsA,
    Object,
    PackType,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use gtk::WindowType::Toplevel;
use relm::{Component, ContainerWidget, RemoteRelm, Widget};
use relm::gtk_ext::BoxExtManual;

use self::Msg::*;

#[derive(Msg)]
pub enum ButtonMsg {
}

#[derive(Clone)]
struct Button {
    button: gtk::Button,
}

impl Widget for Button {
    type Container = gtk::Button;
    type Model = ();
    type Msg = ButtonMsg;

    fn container(&self) -> &Self::Container {
        &self.button
    }

    fn model() -> () {
    }

    fn on_add<W: IsA<gtk::Widget> + IsA<Object>>(&self, parent: W) {
        let parent: gtk::Box = parent
            .upcast::<gtk::Widget>()
            .downcast()
            .expect("Button widget must be added in a gtk::Box");
        parent.set_child_expand(&self.button, false);
        parent.set_child_fill(&self.button, true);
        parent.set_child_pack_type(&self.button, PackType::Start);
        parent.set_child_padding(&self.button, 10);
        parent.set_child_position(&self.button, 0);
    }

    fn update(&mut self, _msg: ButtonMsg, _model: &mut ()) {
    }

    fn view(_relm: RemoteRelm<ButtonMsg>, _model: &Self::Model) -> Self {
        let button = gtk::Button::new_with_label("+");

        Button {
            button: button,
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
    window: gtk::Window,
}

impl Widget for Win {
    type Container = gtk::Window;
    type Model = ();
    type Msg = Msg;

    fn container(&self) -> &Self::Container {
        &self.window
    }

    fn model() -> () {
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: RemoteRelm<Msg>, _model: &Self::Model) -> Self {
        let window = gtk::Window::new(Toplevel);
        let vbox = gtk::Box::new(Vertical, 0);
        window.add(&vbox);
        let label = gtk::Label::new(Some("0"));
        vbox.add(&label);
        let button = gtk::Button::new_with_label("-");
        vbox.add(&button);
        let relm_button = vbox.add_widget::<Button, _>(&relm);
        connect!(relm, window, connect_delete_event(_, _) (Some(Msg::Quit), Inhibit(false)));
        window.show_all();

        Win {
            button: relm_button,
            window: window,
        }
    }
}

fn main() {
    relm::run::<Win>().unwrap();
}

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

use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    ContainerExt,
    Frame,
    Inhibit,
    Label,
    WidgetExt,
    Window,
};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::WindowType::Toplevel;
use relm::{Cast, Component, Container, ContainerWidget, Relm, RelmContainer, Update, Widget};

use self::Msg::*;

struct CenterButton {
    button: gtk::Button,
}

impl Update for CenterButton {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, _msg: ()) {
    }
}

impl Widget for CenterButton {
    type Root = gtk::Button;

    fn parent_id() -> Option<&'static str> {
        Some("center")
    }

    fn root(&self) -> Self::Root {
        self.button.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Rc<RefCell<Self>> {
        let button = gtk::Button::new_with_label("-");
        Rc::new(RefCell::new(CenterButton {
            button: button,
        }))
    }
}

struct Button {
    button: gtk::Button,
}

impl Update for Button {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, _msg: ()) {
    }
}

impl Widget for Button {
    type Root = gtk::Button;

    fn parent_id() -> Option<&'static str> {
        Some("right")
    }

    fn root(&self) -> Self::Root {
        self.button.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Rc<RefCell<Self>> {
        let button = gtk::Button::new_with_label("+");
        Rc::new(RefCell::new(Button {
            button: button,
        }))
    }
}

struct MyFrame {
    frame: Frame,
}

impl Update for MyFrame {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, _msg: ()) {
    }
}

impl Widget for MyFrame {
    type Root = Frame;

    fn root(&self) -> Self::Root {
        self.frame.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Rc<RefCell<Self>> {
        let frame = Frame::new(None);
        Rc::new(RefCell::new(MyFrame {
            frame,
        }))
    }
}

impl Container for MyFrame {
    type Container = Frame;

    fn container(&self) -> &Self::Container {
        &self.frame
    }
}

struct SplitBox {
    hbox1: gtk::Box,
    hbox2: Frame,
    hbox3: Component<MyFrame>,
    vbox: gtk::Box,
}

impl Container for SplitBox {
    type Container = gtk::Box;

    fn container(&self) -> &Self::Container {
        &self.hbox1
    }

    fn add_widget<WIDGET: Widget>(&self, widget: &WIDGET) -> gtk::Container {
        if WIDGET::parent_id() == Some("right") {
            self.hbox3.add(&widget.root());
            self.hbox3.widget().root().clone().upcast()
        }
        else if WIDGET::parent_id() == Some("center") {
            self.hbox2.add(&widget.root());
            self.hbox2.clone().upcast()
        }
        else {
            self.hbox1.add(&widget.root());
            self.hbox1.clone().upcast()
        }
    }
}

impl Update for SplitBox {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn update(&mut self, _event: ()) {
    }
}

impl Widget for SplitBox {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Rc<RefCell<Self>> {
        let vbox = gtk::Box::new(Horizontal, 0);
        let hbox1 = gtk::Box::new(Vertical, 0);
        vbox.add(&hbox1);
        let hbox2 = Frame::new(None);
        vbox.add(&hbox2);
        let hbox3 = vbox.add_widget::<MyFrame, _>(relm, ());
        Rc::new(RefCell::new(SplitBox {
            hbox1,
            hbox2,
            hbox3,
            vbox,
        }))
    }
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

struct Win {
    _button: Component<Button>,
    _center_button: Component<CenterButton>,
    _vbox: Component<SplitBox>,
    window: Window,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Rc<RefCell<Self>> {
        let window = Window::new(Toplevel);
        let vbox = window.add_widget::<SplitBox, _>(relm, ());
        let plus_button = gtk::Button::new_with_label("+");
        vbox.add(&plus_button);
        let label = Label::new(Some("0"));
        vbox.add(&label);
        let button = vbox.add_widget::<Button, _>(relm, ());
        let center_button = vbox.add_widget::<CenterButton, _>(relm, ());
        let minus_button = gtk::Button::new_with_label("-");
        vbox.add(&minus_button);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));
        window.show_all();
        Rc::new(RefCell::new(Win {
            _button: button,
            _center_button: center_button,
            _vbox: vbox,
            window: window,
        }))
    }
}

fn main() {
    Win::run(()).unwrap();
}

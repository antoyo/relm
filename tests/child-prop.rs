/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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
extern crate relm_test;

use gtk::{
    BoxExt,
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
use relm::{
    Component,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};

use self::Msg::*;

#[derive(Msg)]
pub enum ButtonMsg {
}

struct Button {
    button: gtk::Button,
}

impl Update for Button {
    type Model = ();
    type ModelParam = ();
    type Msg = ButtonMsg;

    fn model(_: &Relm<Self>, _: ()) -> () {
    }

    fn update(&mut self, _msg: ButtonMsg) {
    }
}

impl Widget for Button {
    type Root = gtk::Button;

    fn root(&self) -> Self::Root {
        self.button.clone()
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

    fn view(_relm: &Relm<Self>, _model: Self::Model) -> Self {
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
    dec_button: gtk::Button,
    inc_button: Component<Button>,
    label: gtk::Label,
    window: gtk::Window,
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
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let window = gtk::Window::new(Toplevel);
        let vbox = gtk::Box::new(Vertical, 0);
        window.add(&vbox);
        let label = gtk::Label::new(Some("0"));
        vbox.add(&label);
        let dec_button = gtk::Button::new_with_label("-");
        vbox.add(&dec_button);
        let relm_button = vbox.add_widget::<Button>(());
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));
        window.show_all();

        Win {
            dec_button,
            inc_button: relm_button,
            label,
            window: window,
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Win;

    fn get_widgets(&self) -> Self::Widgets {
        self.clone()
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::WidgetExt;

    use relm;

    use Win;

    #[test]
    fn button_position() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let inc_button = widgets.inc_button.widget();
        let dec_button = &widgets.dec_button;
        let label = &widgets.label;

        let inc_allocation = inc_button.get_allocation();
        let dec_allocation = dec_button.get_allocation();
        let label_allocation = label.get_allocation();
        assert!(inc_allocation.y < dec_allocation.y);
        // 10 is the padding.
        assert_eq!(dec_allocation.y, inc_allocation.y + inc_allocation.height + 10 + label_allocation.height);
    }
}

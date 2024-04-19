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

use gtk::{
    Frame,
    Inhibit,
    Label,
    Window,
    prelude::ContainerExt,
    prelude::WidgetExt,
};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::WindowType::Toplevel;
use relm::{
    Cast,
    connect,
    Component,
    Container,
    ContainerComponent,
    ContainerWidget,
    Relm,
    Update,
    Widget,
    WidgetTest,
};
use relm_derive::Msg;

use self::Msg::*;

struct CenterButton {
    button: gtk::Button,
}

impl Update for CenterButton {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) {
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

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        let button = gtk::Button::with_label("-");
        CenterButton {
            button,
        }
    }
}

struct Button {
    button: gtk::Button,
}

impl Update for Button {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) {
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

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        let button = gtk::Button::with_label("+");
        Button {
            button,
        }
    }
}

struct MyFrame {
    frame: Frame,
}

impl Update for MyFrame {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) {
    }

    fn update(&mut self, _msg: ()) {
    }
}

impl Widget for MyFrame {
    type Root = Frame;

    fn root(&self) -> Self::Root {
        self.frame.clone()
    }

    fn view(_relm: &Relm<Self>, _model: ()) -> Self {
        let frame = Frame::new(None);
        MyFrame {
            frame,
        }
    }
}

impl Container for MyFrame {
    type Container = Frame;
    type Containers = ();

    fn container(&self) -> &Self::Container {
        &self.frame
    }

    fn other_containers(&self) -> Self::Containers {
    }
}

struct SplitBox {
    hbox1: gtk::Box,
    hbox2: Frame,
    hbox3: ContainerComponent<MyFrame>,
    vbox: gtk::Box,
}

#[derive(Clone)]
struct SplitBoxContainers {
    hbox1: gtk::Box,
    hbox2: Frame,
    hbox3: Frame,
}

impl Container for SplitBox {
    type Container = gtk::Box;
    type Containers = SplitBoxContainers;

    fn container(&self) -> &Self::Container {
        &self.hbox1
    }

    fn other_containers(&self) -> Self::Containers {
        SplitBoxContainers {
            hbox1: self.hbox1.clone(),
            hbox2: self.hbox2.clone(),
            hbox3: self.hbox3.widget().clone(),
        }
    }

    fn add_widget<WIDGET: Widget>(container: &ContainerComponent<Self>, widget: &Component<WIDGET>) -> gtk::Container
    {
        if WIDGET::parent_id() == Some("right") {
            container.containers.hbox3.add(widget.widget());
            container.containers.hbox3.clone().upcast()
        }
        else if WIDGET::parent_id() == Some("center") {
            container.containers.hbox2.add(widget.widget());
            container.containers.hbox2.clone().upcast()
        }
        else {
            container.containers.hbox1.add(widget.widget());
            container.containers.hbox1.clone().upcast()
        }
    }
}

impl Update for SplitBox {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_: &Relm<Self>, _: ()) {
        
    }

    fn update(&mut self, _event: ()) {
    }
}

impl Widget for SplitBox {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(_relm: &Relm<Self>, _model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Horizontal, 0);
        let hbox1 = gtk::Box::new(Vertical, 0);
        vbox.add(&hbox1);
        let hbox2 = Frame::new(None);
        vbox.add(&hbox2);
        let hbox3 = vbox.add_container::<MyFrame>(());
        SplitBox {
            hbox1,
            hbox2,
            hbox3,
            vbox,
        }
    }
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

struct Components {
    _right_button: Component<Button>,
    _center_button: Component<CenterButton>,
    _vbox: ContainerComponent<SplitBox>,
}

#[derive(Clone)]
struct Widgets {
    button1: gtk::Button,
    label: Label,
    button2: gtk::Button,
    center_button: gtk::Button,
    right_button: gtk::Button,
    window: Window,
}

struct Win {
    _components: Components,
    widgets: Widgets,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) {
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
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, _model: ()) -> Self {
        let window = Window::new(Toplevel);
        let vbox = window.add_container::<SplitBox>(());
        let button1 = gtk::Button::with_label("+");
        vbox.add(&button1);
        let label = Label::new(Some("0"));
        vbox.add(&label);
        let button = vbox.add_widget::<Button>(());
        let center_button = vbox.add_widget::<CenterButton>(());
        let button2 = gtk::Button::with_label("-");
        vbox.add(&button2);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));
        window.show_all();
        Win {
            widgets: Widgets {
                button1,
                button2,
                center_button: center_button.widget().clone(),
                label,
                right_button: button.widget().clone(),
                window,
            },
            _components: Components {
                _right_button: button,
                _center_button: center_button,
                _vbox: vbox,
            }
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {
    }

    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::prelude::WidgetExt;

    use crate::Win;

    #[test]
    fn model_params() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let button1 = &widgets.button1;
        let label = &widgets.label;
        let button2 = &widgets.button2;
        let right_button = widgets.right_button;
        let center_button = widgets.center_button;

        let button1_allocation = button1.allocation();
        let label_allocation = label.allocation();
        let button2_allocation = button2.allocation();
        let right_allocation = right_button.allocation();
        let center_allocation = center_button.allocation();

        assert!(button1_allocation.y() < label_allocation.y());
        assert!(label_allocation.y() < button2_allocation.y());
        assert!(button1_allocation.x() < center_allocation.x());
        assert!(center_allocation.x() < right_allocation.x());
        assert!(center_allocation.y() == right_allocation.y());
    }
}

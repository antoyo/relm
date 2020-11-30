/*
 * Copyright (c) 2020 Boucher, Antoni <bouanto@zoho.com>
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

use std::cell::Cell;

use gtk::{BoxExt, ButtonExt, ContainerExt, WidgetExt};
use relm::{ContainerWidget, Widget, connect};
use relm_derive::Msg;

thread_local! {
    static DROP_COUNT: Cell<i32> = Cell::new(0);
}

macro_rules! assert_drop_count {
    ($value:expr) => {
        assert_eq!(DROP_COUNT.with(|drop_count| drop_count.get()), $value);
    };
}

#[derive(Msg)]
pub enum Msg {
    Quit,
    Add,
    Remove,
}

pub struct RelmWidget {
    root: gtk::Window,
    vbox: gtk::Box,
    label: Option<relm::Component<LabelWidget>>,
}

impl Widget for RelmWidget {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &relm::Relm<Self>, _model: Self::Model) -> Self {
        let root = gtk::Window::new(gtk::WindowType::Toplevel);
        root.set_size_request(400, 400);
        connect!(
            relm,
            root,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), gtk::Inhibit(false))
        );

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add(&vbox);

        let add_button = gtk::ButtonBuilder::new().label("Add").build();
        add_button.set_widget_name("add_button");
        connect!(relm, add_button, connect_clicked(_), Msg::Add);
        vbox.pack_start(&add_button, false, false, 0);

        let remove_button = gtk::ButtonBuilder::new().label("Remove").build();
        remove_button.set_widget_name("remove_button");
        connect!(relm, remove_button, connect_clicked(_), Msg::Remove);
        vbox.pack_start(&remove_button, false, false, 0);

        root.show_all();
        RelmWidget {
            root,
            vbox,
            label: None,
        }
    }
}

impl relm::Update for RelmWidget {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_relm: &relm::Relm<Self>, _param: ()) -> Self::Model {
        ()
    }

    fn update(&mut self, e: Msg) {
        match e {
            Msg::Quit => gtk::main_quit(),
            Msg::Add => {
                let vbox = self.vbox.clone();
                self.label.get_or_insert_with(|| {
                    vbox.add_widget::<LabelWidget>(())
                });
                vbox.show_all();
            }
            Msg::Remove => {
                let vbox = self.vbox.clone();
                if let Some(label) = self.label.take() {
                    vbox.remove_widget(label);
                }
            }
        }
    }
}

pub struct Item {
}

impl Drop for Item {
    fn drop(&mut self) {
        DROP_COUNT.with(|drop_count|
            drop_count.set(drop_count.get() + 1)
        );
    }
}

pub struct LabelWidget {
    pub root: gtk::Label,
    pub item: Item,
}

impl relm::Widget for LabelWidget {
    type Root = gtk::Label;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(_relm: &relm::Relm<Self>, _model: Self::Model) -> Self {
        let root = gtk::LabelBuilder::new().label("hello").build();
        LabelWidget {
            root,
            item: Item {},
        }
    }
}

impl relm::Update for LabelWidget {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_relm: &relm::Relm<Self>, _param: ()) -> Self::Model {
    }

    fn update(&mut self, _e: ()) {
    }
}

#[cfg(test)]
mod tests {
    use super::{RelmWidget, DROP_COUNT};

    use gtk_test::{click, find_widget_by_name};

    #[test]
    fn label_change() {
        gtk::init().expect("gtk init");
        let component = relm::create_component::<RelmWidget>(());
        let root = component.widget();
        let add_button = find_widget_by_name(root, "add_button").expect("find button");
        let remove_button = find_widget_by_name(root, "remove_button").expect("find button");
        assert_drop_count!(0);

        click(&add_button);
        click(&remove_button);
        assert_drop_count!(1);

        click(&add_button);
        click(&remove_button);
        assert_drop_count!(2);
    }
}

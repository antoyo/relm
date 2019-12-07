/*
 * Copyright (c) 2019 Boucher, Antoni <bouanto@zoho.com>
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
    GtkMenuItemExt,
    Inhibit,
    MenuShellExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget, connect};
use relm_derive::{Msg, widget};

use self::Msg::*;

pub struct Model {
    relm: Relm<Win>,
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let file_menu = gtk::Menu::new();
        let file_item = gtk::MenuItem::new_with_label("File");
        file_item.set_submenu(Some(&file_menu));
        let quit_item = gtk::MenuItem::new_with_label("Quit");
        self.menubar.append(&file_item);
        file_menu.append(&quit_item);
        self.menubar.show_all();

        connect!(quit_item, connect_activate(_), self.model.relm, Quit);
    }

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="menubar"]
                gtk::MenuBar {
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

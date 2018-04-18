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
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use gtk::{
    CellLayoutExt,
    CellRendererText,
    Inhibit,
    ListStore,
    ListStoreExt,
    ListStoreExtManual,
    ToValue,
    TreeSelection,
    TreeSelectionExt,
    TreeViewColumn,
    TreeViewExt,
    Type,
    WidgetExt,
};
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

#[widget]
impl Widget for TreeView {
    fn init_view(&mut self) {
        let columns = vec![Type::String];
        let model = ListStore::new(&columns);
        let row = model.append();
        model.set_value(&row, 0, &"String".to_value());
        let row = model.append();
        model.set_value(&row, 0, &"Text".to_value());

        let view_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", 0);
        self.tree_view.append_column(&view_column);

        self.tree_view.set_model(Some(&model));
    }

    fn model() -> () {
    }

    fn update(&mut self, _event: Msg) {
    }

    view! {
        #[name="tree_view"]
        gtk::TreeView {
            selection.changed(selection) => SelectionChanged(selection.clone()),
        }
    }
}

pub struct Model {
    visible: bool,
}

#[derive(Clone, Msg)]
pub enum Msg {
    SelectionChanged(TreeSelection),
    Quit,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let columns = vec![Type::String];
        let model = ListStore::new(&columns);
        let row = model.append();
        model.set_value(&row, 0, &"String".to_value());
        let row = model.append();
        model.set_value(&row, 0, &"Text".to_value());

        let view_column = TreeViewColumn::new();
        let cell = CellRendererText::new();
        view_column.pack_start(&cell, true);
        view_column.add_attribute(&cell, "text", 0);
        self.tree_view.append_column(&view_column);

        self.tree_view.set_model(Some(&model));
    }

    fn model() -> Model {
        Model {
            visible: true,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            SelectionChanged(_selection) => println!("selection changed"),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                #[name="tree_view"]
                gtk::TreeView {
                    selection.changed(selection) => SelectionChanged(selection.clone()),
                },
                TreeView {
                    selection.changed(selection) => SelectionChanged(selection.clone()),
                    visible: self.model.visible,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

#[cfg(test)]
mod tests {
    use gtk::{TreeSelectionExt, TreeModelExt, TreeViewExt};

    use relm;
    use Msg::SelectionChanged;

    use Win;

    #[test]
    fn child_event() {
        let (component, widgets) = relm::init_test::<Win>(()).unwrap();
        let tree_view = &widgets.tree_view;

        let selection_observer = observer_new!(component, SelectionChanged(_));

        let selection = tree_view.get_selection();
        let model = tree_view.get_model().expect("model");
        let iter = model.get_iter_first().expect("first row");
        selection.select_iter(&iter);

        observer_wait!(let SelectionChanged(_selection) = selection_observer);
    }
}

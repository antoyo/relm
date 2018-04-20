/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::fs;
use std::io;
use std::path::PathBuf;

use gtk::{
    CellLayoutExt,
    ContainerExt,
    GtkWindowExt,
    Inhibit,
    ListStoreExtManual,
    StaticType,
    TreeModelExt,
    TreeSelectionExt,
    TreeView,
    TreeViewExt,
    WidgetExt,
    Window,
    WindowType
};
use gtk::Orientation::Vertical;
use relm::{Relm, Update, Widget};

// These two constants stand for the columns of the listmodel and the listview
const VALUE_COL: i32 = 0;
const IS_DIR_COL: i32 = 1;

struct Directory {
    current_dir: PathBuf,
}

#[derive(Msg)]
enum Msg {
    ItemSelect,
    Quit,
}

struct Win {
    tree_view: TreeView,
    model: Directory,
    window: Window,
}

impl Update for Win {
    type Model = Directory;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Directory {
        let working_directory = fs::canonicalize(".").expect("Failed to open directory");
        Directory {
            current_dir: working_directory
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::ItemSelect => {
                let selection = self.tree_view.get_selection();
                if let Some((list_model, iter)) = selection.get_selected() {
                    let is_dir: bool = list_model
                        .get_value(&iter, IS_DIR_COL)
                        .get::<bool>()
                        .unwrap();

                    if is_dir {
                        let dir_name = list_model
                            .get_value(&iter, VALUE_COL)
                            .get::<String>()
                            .unwrap();

                        println!("{:?} selected", dir_name);
                        let new_dir = if dir_name == ".." {
                            // Go up parent directory, if it exists
                            self.model.current_dir
                                .parent()
                                .unwrap_or(&self.model.current_dir)
                                .to_owned()
                        } else {
                            self.model.current_dir.join(dir_name)
                        };
                        self.model.current_dir = new_dir;
                        let new_model = create_and_fill_model(&self.model.current_dir).unwrap();

                        self.tree_view.set_model(Some(&new_model));
                    }
                }
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = gtk::Window::new(WindowType::Toplevel);
        let vbox = gtk::Box::new(Vertical, 0);
        let tree_view = gtk::TreeView::new();
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();

        window.set_title("TreeView example file browser");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        column.pack_start(&cell, true);
        // Assiciate view's column with model's id column
        column.add_attribute(&cell, "text", 0);
        tree_view.append_column(&column);

        let store_model = create_and_fill_model(&model.current_dir).unwrap();
        tree_view.set_model(Some(&store_model));

        vbox.add(&tree_view);
        window.add(&vbox);

        window.show_all();

        connect!(relm, tree_view, connect_cursor_changed(_), Msg::ItemSelect);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        Win {
            tree_view,
            model,
            window,
        }
    }
}

fn create_and_fill_model(dir_str: &PathBuf) -> io::Result<gtk::ListStore> {
    // Single row model
    let model = gtk::ListStore::new(&[String::static_type(), bool::static_type()]);

    // Add the parent directory
    model.insert_with_values(None,
                            &[VALUE_COL as u32, IS_DIR_COL as u32],
                            &[&"..", &true]);

    let entry_iter = fs::read_dir(dir_str)?.filter_map(|x| x.ok());
    for entry in entry_iter {
        if let Ok(metadata) = entry.metadata() {

            if let Ok(file_name) = entry.file_name().into_string() {
                let (final_name, is_dir) = if metadata.is_dir() {
                    (format!("{}/", file_name), true)
                } else {
                    (file_name, false)
                };
                model.insert_with_values(None,
                                        &[VALUE_COL as u32, IS_DIR_COL as u32],
                                        &[&final_name, &is_dir]);
            }
        }
    }
    Ok(model)
}


fn main() {
    Win::run(()).unwrap();
}

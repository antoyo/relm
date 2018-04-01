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

extern crate gio;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gio::{
    AppInfo,
    AppLaunchContext,
    CancellableExt,
    File,
    FileExt,
};
use gtk::{
    ButtonExt,
    DialogExt,
    FileChooserAction,
    FileChooserDialog,
    FileChooserExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk_sys::{GTK_RESPONSE_ACCEPT, GTK_RESPONSE_CANCEL};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget};
use relm_attributes::widget;

use self::Msg::*;

const RESPONSE_ACCEPT: i32 = GTK_RESPONSE_ACCEPT as i32;
const RESPONSE_CANCEL: i32 = GTK_RESPONSE_CANCEL as i32;

pub struct Model {
    relm: Relm<Win>,
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    AppError(gtk::Error),
    AppOpened(()),
    FileRead((Vec<u8>, String)),
    OpenApp,
    OpenFile,
    Quit,
    ReadError(gtk::Error),
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            relm: relm.clone(),
            text: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            AppError(error) => println!("Application error: {}", error),
            AppOpened(()) => println!("Application opened"),
            FileRead((content, _)) => {
                println!("Read: {}", String::from_utf8_lossy(&content));
            },
            OpenApp => self.open_app(),
            OpenFile => self.open_file(),
            Quit => gtk::main_quit(),
            ReadError(error) => println!("Read error: {}", error),
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::Button {
                    clicked => OpenFile,
                    label: "Open file",
                },
                gtk::Label {
                    text: &self.model.text,
                },
                gtk::Button {
                    clicked => OpenApp,
                    label: "Open application",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl Win {
    fn open_app(&mut self) {
        let dialog = FileChooserDialog::new(Some("Open a file"), Some(&self.window), FileChooserAction::Open);
        dialog.add_button("Cancel", RESPONSE_CANCEL);
        dialog.add_button("Accept", RESPONSE_ACCEPT);
        let result = dialog.run();
        if result == RESPONSE_ACCEPT {
            if let Some(uri) = dialog.get_uri() {
                let app_launch_context = AppLaunchContext::new();
                //connect_async_func!(AppInfo::launch_default_for_uri_async(&uri, &app_launch_context), self.model.relm, AppOpened);
                let cancellable = connect_async_func_full!(AppInfo::launch_default_for_uri_async(&uri, &app_launch_context), self.model.relm, AppOpened, AppError);
                cancellable.cancel();
            }
        }
        dialog.destroy();
    }

    fn open_file(&mut self) {
        let dialog = FileChooserDialog::new(Some("Open a file"), Some(&self.window), FileChooserAction::Open);
        dialog.add_button("Cancel", RESPONSE_CANCEL);
        dialog.add_button("Accept", RESPONSE_ACCEPT);
        let result = dialog.run();
        if result == RESPONSE_ACCEPT {
            if let Some(filename) = dialog.get_filename() {
                let file = File::new_for_path(filename);
                //connect_async!(file, load_contents_async, self.model.relm, FileRead);
                //let cancellable = connect_async_full!(file, load_contents_async, self.model.relm, FileRead);
                //connect_async!(file, load_contents_async, self.model.relm, FileRead, ReadError);
                let cancellable = connect_async_full!(file, load_contents_async, self.model.relm, FileRead, ReadError);
                cancellable.cancel();
            }
        }
        dialog.destroy();
    }
}

fn main() {
    Win::run(()).unwrap();
}

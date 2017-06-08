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

// TODO: fix futures-glib to support recursive source.

extern crate chrono;
extern crate futures;
extern crate futures_glib;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::time::Duration;

use chrono::Local;
use futures_glib::Interval;
use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Dialog,
    DialogExt,
    Inhibit,
    Label,
    WidgetExt,
    Window,
    WindowType,
    DIALOG_MODAL,
};
use gtk::Orientation::Vertical;
use relm::{Relm, Resolver, Update, Widget};

use Msg::*;

struct Model {
    counter: i32,
    relm: Relm<Win>,
}

#[derive(Msg)]
enum Msg {
    Delete(Resolver<Inhibit>),
    Quit,
    Tick(()),
}

// Create the structure that holds the widgets used in the view.
struct Win {
    label: Label,
    model: Model,
    window: Window,
}

impl Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            relm: relm.clone(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let stream = Interval::new(Duration::from_secs(1));
        relm.connect_exec_ignore_err(stream, Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Delete(mut resolver) => {
                let num = dialog(&self.window);
                match num {
                    1 => self.model.relm.stream().emit(Quit),
                    _ => resolver.resolve(Inhibit(true)),
                }
            },
            Quit => gtk::main_quit(),
            Tick(()) => {
                let time = Local::now();
                self.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
        }
    }
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);

        let label = Label::new(None);
        window.add(&label);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), async Delete);

        let mut win = Win {
            label,
            model,
            window: window,
        };
        win.update(Tick(()));
        win
    }
}

fn dialog(window: &Window) -> i32 {
    let buttons = &[("Yes", 1), ("No", 2)];
    let dialog = Dialog::new_with_buttons(Some("Quit?"), Some(window), DIALOG_MODAL, buttons);
    let result = dialog.run();
    dialog.destroy();
    result
}

fn main() {
    Win::run(()).unwrap();
}

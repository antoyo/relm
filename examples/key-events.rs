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

extern crate futures_glib;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use gtk::{
    Inhibit,
    WidgetExt,
    Window,
    WindowType,
};
use relm::{Relm, Update, Widget};

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Delete,
    Press,
    Release,
    Quit,
}

pub struct Model {
    press_count: i32,
    relm: Relm<Win>,
}

struct Win {
    model: Model,
    window: Window,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            press_count: 0,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Delete => {
                if self.model.press_count <= 3 {
                    self.model.relm.stream().emit(Quit);
                }
            },
            Press => {
                self.model.press_count += 1;
                println!("Press");
            },
            Release => {
                println!("Release");
            },
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Win>, model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);

        window.show_all();

        connect!(relm, window, connect_key_press_event(_, _), return (Press, Inhibit(false)));
        connect!(relm, window, connect_key_release_event(_, _), return (Release, Inhibit(false)));
        connect!(relm, window, connect_delete_event(_, _), return (Delete, Inhibit(true)));

        Win {
            model,
            window,
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

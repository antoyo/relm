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
use relm::{RemoteRelm, Widget};

use self::Msg::*;

#[derive(Msg)]
pub enum Msg {
    Press,
    Release,
    Quit,
}

#[derive(Clone)]
pub struct Model {
    press_count: i32,
}

#[derive(Clone)]
struct Win {
    window: Window,
}

impl Widget for Win {
    type Model = Model;
    type Msg = Msg;
    type Root = Window;

    fn model() -> Model {
        Model {
            press_count: 0,
        }
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Press => {
                model.press_count += 1;
                println!("Press");
            },
            Release => {
                println!("Release");
            },
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &RemoteRelm<Win>, _model: &Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);

        window.show_all();

        connect!(relm, window, connect_key_press_event(_, _) (Press, Inhibit(false)));
        connect!(relm, window, connect_key_release_event(_, _) (Release, Inhibit(false)));
        connect!(relm, window, connect_delete_event(_, _) with model
            Self::quit(model));

        Win {
            window: window,
        }
    }
}

impl Win {
    fn quit(model: &mut Model) -> (Option<Msg>, Inhibit) {
        if model.press_count > 3 {
            (None, Inhibit(true))
        }
        else {
            (Some(Quit), Inhibit(false))
        }
    }
}

fn main() {
    relm::run::<Win>().unwrap();
}

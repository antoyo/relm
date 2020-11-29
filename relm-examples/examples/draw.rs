/*
 * Copyright (c) 2019-2020 Boucher, Antoni <bouanto@zoho.com>
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
use std::rc::Rc;

use gdk::EventKey;
use gtk::{
    Inhibit,
    WidgetExt,
};
use relm::Widget;
use relm_derive::{Msg, widget};

use self::Msg::*;

pub struct Model {
    letter: Rc<Cell<char>>,
}

#[derive(Msg)]
pub enum Msg {
    KeyPress(EventKey),
    Quit,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        let letter = self.model.letter.clone();
        self.widgets.drawing_area.connect_draw(move |_, context| {
            context.set_source_rgb(0.2, 0.4, 0.0);
            context.paint();

            context.set_font_size(60.0);
            context.set_source_rgb(0.0, 0.0, 0.0);
            context.move_to(100.0, 100.0);
            context.show_text(&letter.get().to_string());
            Inhibit(false)
        });
    }

    fn model() -> Model {
        Model {
            letter: Rc::new(Cell::new(' ')),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            KeyPress(event) => {
                if let Some(letter) = event.get_keyval().to_unicode() {
                    self.model.letter.set(letter);
                    self.widgets.drawing_area.queue_draw();
                }
            },
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            #[name="drawing_area"]
            gtk::DrawingArea {
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
            key_press_event(_, event) => (KeyPress(event.clone()), Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

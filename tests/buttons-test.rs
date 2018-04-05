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
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget};
use relm_attributes::widget;

use self::Msg::*;

#[derive(Clone)]
pub struct Model {
    counter: i32,
    relm: Relm<Win>,
}

#[derive(Msg)]
pub enum Msg {
    Decrement,
    FiveInc,
    GetModel,
    RecvModel(Model),
    Increment,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            // To be listened to by the user.
            FiveInc => (),
            GetModel => self.model.relm.stream().emit(RecvModel(self.model.clone())),
            Increment => {
                self.model.counter += 1;
                if self.model.counter == 5 {
                    self.model.relm.stream().emit(FiveInc);
                }
            },
            // To be listened to by the user.
            RecvModel(_) => (),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="inc_button"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                #[name="dec_button"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use gtk;
    use gtk::LabelExt;

    use relm;
    use relm_test::click;

    //use Msg::FiveInc;
    use Msg::{GetModel, RecvModel};
    use Win;

    #[test]
    fn label_change() {
        let (component, widgets) = relm::init_test::<Win>(()).unwrap();
        let inc_button = widgets.inc_button.clone();
        let dec_button = widgets.dec_button.clone();

        assert_text!(widgets.label, 0);
        click(&inc_button);
        assert_text!(widgets.label, 1);
        click(&inc_button);
        assert_text!(widgets.label, 2);
        click(&dec_button);
        assert_text!(widgets.label, 1);
        click(&dec_button);
        assert_text!(widgets.label, 0);
        click(&dec_button);
        assert_text!(widgets.label, -1);

        // Observe for events on the widget.
        component.stream().observe(|msg| {
            match msg {
                /*FiveInc => {
                    // Hack to avoid exiting the main loop too quickly.
                    gtk::timeout_add(10, || {
                        gtk::main_quit();
                        gtk::Continue(false)
                    });
                },*/
                RecvModel(model) => {
                    assert_eq!(model.counter, 5);
                    // Allow the test to finish.
                    gtk::main_quit();
                },
                _ => (),
            }
        });

        for _ in 0..6 {
            click(&inc_button);
        }

        // Ask for the model. This will emit RecvModel.
        component.stream().emit(GetModel);

        // Prevent the test from finishing early.
        gtk::main();
    }
}

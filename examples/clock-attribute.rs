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

#![feature(proc_macro, unboxed_closures)]

extern crate chrono;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate relm_test;

use chrono::{DateTime, Local};
use gtk::{
    Inhibit,
    LabelExt,
    WidgetExt,
};
use relm::{Relm, Widget, interval};
use relm_attributes::widget;

use self::Msg::*;

pub struct Model {
    time: DateTime<Local>,
}

#[derive(Msg)]
pub enum Msg {
    Quit,
    Tick,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            time: Local::now(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Tick => self.model.time = Local::now(),
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            #[name="label"]
            gtk::Label {
                text: &self.model.time.format("%H:%M:%S").to_string(),
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
    use chrono::{Local, NaiveTime};
    use gtk::LabelExt;

    use relm;
    use relm_test::wait;

    use Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).unwrap();
        let label = &widgets.label;

        fn time_close(time1: String, time2: String) -> bool {
            println!("{}", time1);
            println!("{}", time2);
            let date1 = NaiveTime::parse_from_str(&time1, "%H:%M:%S").expect("parse time1");
            let date2 = NaiveTime::parse_from_str(&time2, "%H:%M:%S").expect("parse time2");
            (date1.signed_duration_since(date2)).num_seconds() <= 1
        }

        let time = Local::now();
        assert!(time_close(label.get_text().expect("text"), time.format("%H:%M:%S").to_string()));

        wait(2000);

        let time2 = Local::now();
        assert_ne!(time, time2);
        assert!(time_close(label.get_text().expect("text"), time2.format("%H:%M:%S").to_string()));
    }
}

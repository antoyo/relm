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

extern crate chrono;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate relm_test;

use chrono::Local;
use gtk::{
    ContainerExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
};
use relm::{Relm, Update, Widget, WidgetTest, interval};

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Quit,
    Tick,
}

#[derive(Clone)]
struct Win {
    label: Label,
    window: Window,
}

impl Update for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> () {
        ()
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Tick => {
                let time = Local::now();
                self.label.set_text(&format!("{}", time.format("%H:%M:%S")));
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

    fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
        let label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        window.add(&label);

        window.show_all();

        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        let mut win = Win {
            label: label,
            window: window,
        };

        win.update(Tick);
        win
    }
}

impl WidgetTest for Win {
    type Widgets = Win;

    fn get_widgets(&self) -> Self::Widgets {
        self.clone()
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

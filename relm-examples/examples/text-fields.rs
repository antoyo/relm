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

use gtk::{
    ContainerExt,
    EditableSignals,
    Entry,
    EntryExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::Vertical;
use relm_derive::Msg;
use relm::{connect, Relm, Update, Widget, WidgetTest};

use self::Msg::*;

struct Model {
    content: String,
}

#[derive(Msg)]
enum Msg {
    Change,
    Quit,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

#[derive(Clone)]
struct Widgets {
    input: Entry,
    label: Label,
    window: Window,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            content: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Change => {
                self.model.content = self.widgets.input.get_text()
                                                       .expect("get_text failed")
                                                       .chars()
                                                       .rev()
                                                       .collect();
                self.widgets.label.set_text(&self.model.content);
            },
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, input, connect_changed(_), Change);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Quit), Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                input,
                label,
                window,
            },
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gdk::enums::key;
    use gtk::LabelExt;

    use gtk_test::{assert_text, enter_key, enter_keys};

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let entry = &widgets.input;
        let label = &widgets.label;

        assert_text!(label, "");

        enter_keys(entry, "test");
        assert_text!(label, "tset");

        enter_key(entry, key::BackSpace);
        assert_text!(label, "set");

        enter_key(entry, key::Home);
        //enter_key(entry, key::Delete); // TODO: when supported by enigo.
        enter_keys(entry, "a");
        assert_text!(label, "seta");

        enter_key(entry, key::End);
        enter_keys(entry, "a");
        //assert_text!(label, "aseta"); // FIXME
    }
}

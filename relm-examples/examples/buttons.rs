/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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
    Button,
    Inhibit,
    Label,
    Window,
    WindowType,
    prelude::ButtonExt,
    prelude::ContainerExt,
    prelude::LabelExt,
    prelude::WidgetExt,
};
use gtk::Orientation::Vertical;
use relm_derive::Msg;
use relm::{connect, Relm, Update, Widget, WidgetTest};

struct Model {
    counter: i32,
}

#[derive(Msg)]
enum Msg {
    Decrement,
    Increment,
    Quit,
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    counter_label: Label,
    minus_button: Button,
    plus_button: Button,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        let label = &self.widgets.counter_label;

        match event {
            Msg::Decrement => {
                self.model.counter -= 1;
                // Manually update the view.
                label.set_text(&self.model.counter.to_string());
            },
            Msg::Increment => {
                self.model.counter += 1;
                label.set_text(&self.model.counter.to_string());
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // Create the view using the normal GTK+ method calls.
        let vbox = gtk::Box::new(Vertical, 0);

        let plus_button = Button::with_label("+");
        vbox.add(&plus_button);

        let counter_label = Label::new(Some("0"));
        vbox.add(&counter_label);

        let minus_button = Button::with_label("-");
        vbox.add(&minus_button);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        // Send the message Increment when the button is clicked.
        connect!(relm, plus_button, connect_clicked(_), Msg::Increment);
        connect!(relm, minus_button, connect_clicked(_), Msg::Decrement);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                counter_label,
                minus_button,
                plus_button,
                window: window,
            },
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {
    }

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
    use gtk::prelude::LabelExt;

    use gtk_test::assert_text;
    use relm_test::click;

    use crate::Win;

    #[test]
    fn label_change() {
        let (_component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let plus_button = &widgets.plus_button;
        let minus_button = &widgets.minus_button;
        let label = &widgets.counter_label;

        assert_text!(label, 0);
        click(plus_button);
        assert_text!(label, 1);
        click(plus_button);
        assert_text!(label, 2);
        click(plus_button);
        assert_text!(label, 3);
        click(plus_button);
        assert_text!(label, 4);

        click(minus_button);
        assert_text!(label, 3);
        click(minus_button);
        assert_text!(label, 2);
        click(minus_button);
        assert_text!(label, 1);
        click(minus_button);
        assert_text!(label, 0);
        click(minus_button);
        assert_text!(label, -1);
    }
}

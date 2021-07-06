use relm_derive::widget;

#[widget]
impl Widget for Foo {
    fn model() {}

    view! {
        gtk::Window {}
    }
}

fn main() {}

use relm_derive::widget;

#[widget]
impl Widget for Foo {
    fn update(&mut self, model: ()) {}

    view! {
        gtk::Window {}
    }
}

fn main() {}

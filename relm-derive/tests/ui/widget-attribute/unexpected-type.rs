use relm_derive::widget;

#[widget]
impl Widget for Foo {
    type Unexpected = i32;
}

fn main() {}

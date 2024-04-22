#![allow(unused_imports)]

use relm::Widget;
use relm_derive::widget;

#[widget]
impl Widget for Foo {
    fn model() {}

    fn update(&mut self, _: ()) {}

    view! {}
}

fn main() {}

/*
 * Copyright (c) 2020 Boucher, Antoni <bouanto@zoho.com>
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

use relm::{Update, Widget};

thread_local! {
    static DROPPED: Cell<bool> = Cell::new(false);
}

pub struct Item {
}

impl Drop for Item {
    fn drop(&mut self) {
        DROPPED.with(|dropped|
            dropped.set(true)
        );
    }
}

pub struct RelmWidget {
    pub root: gtk::Label,
    pub item: Item,
}

impl Widget for RelmWidget {
    type Root = gtk::Label;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(_relm: &relm::Relm<Self>, _model: Self::Model) -> Self {
        let root = gtk::LabelBuilder::new().label("hello").build();
        RelmWidget {
            root,
            item: Item {},
        }
    }
}

impl Update for RelmWidget {
    type Model = ();
    type ModelParam = ();
    type Msg = ();

    fn model(_relm: &relm::Relm<Self>, _param: ()) -> Self::Model {
    }

    fn update(&mut self, _e: ()) {
    }
}

#[cfg(test)]
mod tests {
    use super::{RelmWidget, DROPPED};

    #[test]
    fn label_change() {
        gtk::init().expect("gtk init");
        {
            let _component = relm::create_component::<RelmWidget>(());
        }
        assert!(DROPPED.with(|dropped| dropped.get()));
    }
}

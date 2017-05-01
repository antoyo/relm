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

use std::cell::{Ref, RefCell};
use std::rc::Rc;

use super::{EventStream, Widget};

/// Widget that was added by the `ContainerWidget::add_widget()` method.
///
/// ## Warning
/// You must keep your components as long as you want them to send/receive events.
/// Common practice is to store `Component`s in the `Widget` struct (see the [communication
/// example](https://github.com/antoyo/relm/blob/master/examples/communication.rs#L182-L188)).
/// The `#[widget]` attribute takes care of storing them in the struct automatically (see the
/// [communication-attribute example](https://github.com/antoyo/relm/blob/master/examples/communication-attribute.rs)).
#[must_use]
#[derive(Clone)]
pub struct Component<WIDGET: Widget> {
    stream: EventStream<WIDGET::Msg>,
    widget: Rc<RefCell<WIDGET>>,
}

impl<WIDGET: Widget> Drop for Component<WIDGET> {
    fn drop(&mut self) {
        let _ = self.stream.close();
    }
}

impl<WIDGET: Widget> Component<WIDGET> {
    #[doc(hidden)]
    pub fn new(stream: EventStream<WIDGET::Msg>, widget: Rc<RefCell<WIDGET>>) -> Self {
        Component {
            stream,
            widget,
        }
    }

    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<WIDGET::Msg> {
        &self.stream
    }

    /// Get the widget of this component.
    pub fn widget(&self) -> Ref<WIDGET> {
        // TODO: check if we can do better than returning the result of borrow.
        self.widget.borrow()
    }

    /// Get the ref-counted widget of this component.
    pub fn widget_rc(&self) -> &Rc<RefCell<WIDGET>> {
        &self.widget
    }
}

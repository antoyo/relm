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

use super::{
    EventStream,
    StreamHandle,
    Widget,
};

/// Widget that was added by the `ContainerWidget::add_widget()` method.
///
/// ## Warning
/// You must keep your components as long as you want them to send/receive events.
/// Common practice is to store `Component`s in the `Widget` struct (see the [communication
/// example](https://github.com/antoyo/relm/blob/master/tests/communication.rs#L216-L220)).
/// The `#[widget]` attribute takes care of storing them in the struct automatically (see the
/// [communication-attribute example](https://github.com/antoyo/relm/blob/master/tests/communication-attribute.rs)).
#[must_use]
pub struct Component<WIDGET: Widget> {
    stream: EventStream<WIDGET::Msg>,
    widget: WIDGET::Root,
}

/*impl<WIDGET: Widget> Clone for Component<WIDGET> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            widget: self.widget.clone(),
        }
    }
}*/

impl<WIDGET: Widget> Component<WIDGET> {
    #[doc(hidden)]
    pub fn new(stream: EventStream<WIDGET::Msg>, widget: WIDGET::Root) -> Self {
        Component {
            stream,
            widget,
        }
    }

    /// Emit a message of the widget stream.
    pub fn emit(&self, msg: WIDGET::Msg) {
        self.stream.emit(msg);
    }

    /// Get the event stream of the component.
    /// This is used internally by the library.
    pub fn stream(&self) -> StreamHandle<WIDGET::Msg> {
        self.stream.downgrade()
    }

    /// Get the event stream of the component.
    /// This is used internally by the library.
    pub fn owned_stream(&self) -> &EventStream<WIDGET::Msg> {
        &self.stream
    }

    /// Get the widget of the component.
    pub fn widget(&self) -> &WIDGET::Root {
        &self.widget
    }
}

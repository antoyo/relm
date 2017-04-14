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

use gtk::{self, IsA};

use super::{DisplayVariant, Relm, RemoteRelm};

/// Trait to implement to manage widget's events.
pub trait Widget<MSG: Clone + DisplayVariant>
    where Self: Sized,
          Self::Container: Clone + IsA<gtk::Widget>,
{
    /// The type of the containing widget.
    type Container;
    /// The type of the model.
    type Model;

    /// Get the containing widget, i.e. the parent widget of the view.
    fn container(&self) -> &Self::Container;

    /// Create the initial model.
    fn model() -> Self::Model;

    /// Connect the subscriptions.
    /// Subscriptions are `Future`/`Stream` that are spawn when the widget is created.
    ///
    /// ## Note
    /// This method is called in the tokio thread, so that you can spawn `Future`s and `Stream`s.
    fn subscriptions(_relm: &Relm<MSG>) {
    }

    /// Method called when a message is received from an event.
    ///
    /// ## Note
    /// This method is called in the GTK+ thread, so that you can update widgets.
    fn update(&mut self, event: MSG, model: &mut Self::Model);

    /// Connect `Future`s or `Stream`s when receiving an event.
    ///
    /// ## Warning
    /// This method is executed in the tokio thread: hence, you **must** spawn any futures in this
    /// method, not in `Widget::update()`.
    fn update_command(_relm: &Relm<MSG>, _event: MSG, _model: &mut Self::Model) {
    }

    /// Create the initial view.
    fn view(relm: RemoteRelm<MSG>, model: &Self::Model) -> Self;
}

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

use gtk::{self, IsA, Object};

use super::{DisplayVariant, Relm, RemoteRelm};

/// Trait to implement to manage widget's events.
pub trait Widget
    where Self: Clone,
          Self::Root: Clone + IsA<gtk::Widget>,
          Self::Msg: Clone + DisplayVariant,
{
    /// The type of the model.
    type Model;
    /// The type of the messages sent to the [`update()`](trait.Widget.html#tymethod.update) method.
    type Msg;
    /// The type of the root widget.
    type Root;

    /// Update the view after it is initially created.
    /// This method is only useful when using the `#[widget]` attribute, because when not using it,
    /// you can use the [`view()`](trait.Widget.html#tymethod.view) method instead.
    fn init_view(&self) {
    }

    /// Create the initial model.
    fn model() -> Self::Model;

    /// Method called when the widget is added to its parent.
    fn on_add<W: IsA<gtk::Widget> + IsA<Object>>(&self, _parent: W) {
    }

    /// Get the root widget of the view.e. the root widget of the view.
    fn root(&self) -> &Self::Root;

    /// Connect the subscriptions.
    /// Subscriptions are `Future`/`Stream` that are spawn when the widget is created.
    ///
    /// ## Note
    /// This method is called in the tokio thread, so that you can spawn `Future`s and `Stream`s.
    fn subscriptions(_relm: &Relm<Self::Msg>) {
    }

    /// Method called when a message is received from an event.
    ///
    /// ## Note
    /// This method is called in the GTK+ thread, so that you can update widgets.
    fn update(&mut self, event: Self::Msg, model: &mut Self::Model);

    /// Connect `Future`s or `Stream`s when receiving an event.
    ///
    /// ## Warning
    /// This method is executed in the tokio thread: hence, you **must** spawn any futures in this
    /// method, not in `Widget::update()`.
    fn update_command(_relm: &Relm<Self::Msg>, _event: Self::Msg, _model: &mut Self::Model) {
    }

    /// Create the initial view.
    fn view(relm: RemoteRelm<Self::Msg>, model: &Self::Model) -> Self;
}

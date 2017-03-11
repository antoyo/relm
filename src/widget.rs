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

use super::{DisplayVariant, Relm, UnitFuture};

/// Trait to implement to manage widget's events.
pub trait Widget<M: Clone + DisplayVariant>
    where Self::Container: Clone + IsA<gtk::Widget>
{
    type Container;

    /// Get the containing widget.
    fn container(&self) -> &Self::Container;

    /// Create the widget.
    fn new(relm: Relm<M>) -> Self;

    /// Get the default subscriptions.
    fn subscriptions(&self) -> Vec<UnitFuture> {
        vec![]
    }

    /// Method called when a message is received from an event.
    fn update(&mut self, event: M) -> UnitFuture;
}

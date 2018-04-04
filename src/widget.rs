/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
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

use super::{Relm, run};
use relm_state::Update;

/// Trait to implement to manage widget's events.
pub trait Widget
    where Self: Update,
          Self::Root: Clone + IsA<Object> + IsA<gtk::Widget>,
{
    /// The type of the root widget.
    type Root;

    #[cfg(feature = "test")]
    /// Represents the structure holding all the widgets. Useful for tests.
    type Widgets;

    #[cfg(feature = "test")]
    /// Get the structure containing all the widgets. Useful for tests.
    fn get_widgets(&self) -> Self::Widgets;

    /// Update the view after it is initially created.
    /// This method is only useful when using the `#[widget]` attribute, because when not using it,
    /// you can use the [`view()`](trait.Widget.html#tymethod.view) method instead.
    fn init_view(&mut self) {
    }

    /// Method called when the widget is added to its parent.
    fn on_add<W: IsA<gtk::Widget> + IsA<Object>>(&self, _parent: W) {
    }

    /// Get the parent ID.
    /// This is useful for custom Container implementation: when you implement the
    /// [`Container::add_widget()`](trait.Container.html#tymethod.add_widget), you might want to
    /// insert widgets elsewhere depending of this id.
    fn parent_id() -> Option<&'static str> {
        None
    }

    // TODO: ajouter une méthode param() pour déterminer des paramètres qui seront pris en compte à
    // l’ajout du widget.

    /// Get the root widget of the view.
    fn root(&self) -> Self::Root;

    /// Create the window from this widget and start the main loop.
    fn run(model_param: Self::ModelParam) -> Result<(), ()>
        where Self: 'static,
    {
        run::<Self>(model_param)
    }

    /// Create the initial view.
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self;
}

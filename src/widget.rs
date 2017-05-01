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

use super::{DisplayVariant, Relm};

/// Trait to implement to manage widget's events.
pub trait Widget
    where Self: Clone,
          Self::Root: Clone + IsA<gtk::Widget>,
          Self::Msg: Clone + DisplayVariant,
{
    /// The type of the model.
    type Model;
    /// The type of the parameter of the model() function used to initialize the model.
    type ModelParam: Sized;
    /// The type of the messages sent to the [`update()`](trait.Widget.html#tymethod.update) method.
    type Msg;
    /// The type of the root widget.
    type Root;

    /// Update the view after it is initially created.
    /// This method is only useful when using the `#[widget]` attribute, because when not using it,
    /// you can use the [`view()`](trait.Widget.html#tymethod.view) method instead.
    fn init_view(&self, _model: &mut Self::Model) {
    }

    /// Create the initial model.
    fn model(param: Self::ModelParam) -> Self::Model;

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

    /// Get the root widget of the view.e. the root widget of the view.
    fn root(&self) -> &Self::Root;

    /// Create the window from this widget and start the main loop.
    fn run(model_param: Self::ModelParam) -> Result<(), ()>
        where Self: 'static,
              Self::Model: Clone,
              Self::ModelParam: Default,
    {
        run::<Self>(model_param)
    }

    /// Connect the subscriptions.
    /// Subscriptions are `Future`/`Stream` that are spawn when the widget is created.
    fn subscriptions(_relm: &Relm<Self::Msg>) {
    }

    /// Method called when a message is received from an event.
    fn update(&mut self, event: Self::Msg, model: &mut Self::Model);

    /// Create the initial view.
    fn view(relm: Relm<Self::Msg>, model: &Self::Model) -> Self;
}

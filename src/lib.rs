/*
 * Copyright (c) 2017-2019 Boucher, Antoni <bouanto@zoho.com>
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

//! Asynchronous GUI library based on GTK+.
//!
//! This library provides a [`Widget`](trait.Widget.html) trait that you can use to create asynchronous GUI components.
//! This is the trait you will need to implement for your application.
//! It helps you to implement MVC (Model, View, Controller) in an elegant way.
//!
//! ## Installation
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! gtk = "^0.6.0"
//! relm = "^0.16.0"
//! relm-derive = "^0.16.0"
//! ```
//!
//! More info can be found in the [readme](https://github.com/antoyo/relm#relm).

#![allow(clippy::new_without_default)]

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
)]

/*
 * TODO: allow using self in the event connection right side (messages to be sent) to remove the
 * with syntax.
 *
 * TODO: improve README so that examples can be copy/pasted.
 *
 * FIXME: some relm widgets requires { and } (see the rusic music-player) while other do not.
 * FIXME: should not require to import WidgetExt because it calls show().
 * FIXME: cannot have relm event with tuple as a value like:
 * RelmWidget {
 *     RelmEvent((value1, value2)) => …
 * }
 * TODO: use pub(crate) instead of pub so that we're not bound to make the model and msg structs pub.
 *
 * TODO: add init() method to the Widget (or Update) trait as a shortcut for init::<Widget>()?
 *
 * TODO: show a warning when a component is imediately destroyed.
 * FIXME: cannot add a trailing coma at the end of a initializer list.
 * TODO: switch from gtk::main() to MainLoop to avoid issues with nested loops.
 * TODO: prefix generated container name with _ to hide warnings.
 * TODO: remove the code generation related to using self in event handling.
 * TODO: remove the closure transformer code.
 *
 * TODO: move most of the examples in the tests/ directory.
 * TODO: add construct-only properties for relm widget (to replace initial parameters) to allow
 * setting them by name (or with default value).
 * TODO: find a way to do two-step initialization (to avoid using unitialized in model()).
 * TODO: consider using GBinding instead of manually adding calls to set_property().
 *
 * TODO: add a FAQ with the question related to getting an error when not importing the gtk traits.
 *
 * TODO: show a warning when components are destroyed after the end of call to Widget::view().
 * TODO: warn in the attribute when an event cycle is found.
 * TODO: add a Deref<Widget> for Component?
 * TODO: look at how Elm works with the <canvas> element.
 * TODO: the widget names should start with __relm_field_.
 *
 * TODO: reset widget name counters when creating new widget?
 *
 * TODO: refactor the code.
 *
 * TODO: chat client/server example.
 *
 * TODO: add default type of () for Model in Widget when it is stable.
 * TODO: optionnaly multi-threaded.
 */

mod component;
mod container;
mod core;
mod drawing;
mod macros;
mod state;
mod widget;

#[doc(hidden)]
pub use fragile::Fragile;

#[doc(hidden)]
pub use glib::{
    Cast,
    IsA,
    Object,
    StaticType,
    ToValue,
    Value,
};
#[doc(hidden)]
pub use glib::translate::{FromGlibPtrNone, IntoGlib, ToGlibPtr};
#[doc(hidden)]
pub use gobject_sys::{GParameter, g_object_newv};
use glib::Continue;

pub use crate::core::{Channel, EventStream, Sender, StreamHandle};
pub use crate::state::{
    DisplayVariant,
    IntoOption,
    IntoPair,
    Relm,
    Update,
    UpdateNew,
    execute,
};
use state::init_component;

pub use component::Component;
pub use container::{Container, ContainerComponent, ContainerWidget};
pub use drawing::DrawHandler;
pub use widget::{Widget, WidgetTest};

/// Dummy macro to be used with `#[derive(Widget)]`.
#[macro_export]
macro_rules! impl_widget {
    ($($tt:tt)*) => {
        ()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! use_impl_self_type {
    (impl $(::relm::)*Widget for $self_type:ident { $($tts:tt)* }) => {
        pub use self::__relm_gen_private::$self_type;
    };
}

fn create_widget_test<WIDGET>(model_param: WIDGET::ModelParam) -> (Component<WIDGET>, WIDGET::Streams, WIDGET::Widgets)
    where WIDGET: Widget + WidgetTest + 'static,
          WIDGET::Msg: DisplayVariant + 'static,
{
    let (component, widget, relm) = create_widget::<WIDGET>(model_param);
    let widgets = widget.get_widgets();
    let streams = widget.get_streams();
    init_component::<WIDGET>(component.owned_stream(), widget, &relm);
    (component, streams, widgets)
}

/// Create a new relm widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_component<CHILDWIDGET>(model_param: CHILDWIDGET::ModelParam)
        -> Component<CHILDWIDGET>
    where CHILDWIDGET: Widget + 'static,
          CHILDWIDGET::Msg: DisplayVariant + 'static,
{
    let (component, widget, child_relm) = create_widget::<CHILDWIDGET>(model_param);
    init_component::<CHILDWIDGET>(component.owned_stream(), widget, &child_relm);
    component
}

/// Create a new relm container widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_container<CHILDWIDGET>(model_param: CHILDWIDGET::ModelParam)
        -> ContainerComponent<CHILDWIDGET>
    where CHILDWIDGET: Container + Widget + 'static,
          CHILDWIDGET::Msg: DisplayVariant + 'static,
{
    let (component, widget, child_relm) = create_widget::<CHILDWIDGET>(model_param);
    let container = widget.container().clone();
    let containers = widget.other_containers();
    init_component::<CHILDWIDGET>(component.owned_stream(), widget, &child_relm);
    ContainerComponent::new(component, container, containers)
}

/// Create a new relm widget with `model_param` as initialization value.
fn create_widget<WIDGET>(model_param: WIDGET::ModelParam)
    -> (Component<WIDGET>, WIDGET, Relm<WIDGET>)
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static,
{
    let stream = EventStream::new();

    let relm = Relm::new(&stream);
    let model = WIDGET::model(&relm, model_param);
    let mut widget = WIDGET::view(&relm, model);
    widget.init_view();

    let root = widget.root();
    (Component::new(stream, root), widget, relm)
}

type InitTestComponents<WIDGET> = (Component<WIDGET>, <WIDGET as WidgetTest>::Streams, <WIDGET as WidgetTest>::Widgets);

/// Initialize a widget for a test.
///
/// It is to be used this way:
///
/// ```
/// # use gtk::{Window, WindowType};
/// # use relm::{Relm, Update, Widget, WidgetTest};
/// # use relm_derive::Msg;
/// #
/// # #[derive(Clone)]
/// # struct Win {
/// #     window: Window,
/// # }
/// #
/// # impl Update for Win {
/// #     type Model = ();
/// #     type ModelParam = ();
/// #     type Msg = Msg;
/// #
/// #     fn model(_: &Relm<Self>, _: ()) -> () {
/// #         ()
/// #     }
/// #
/// #     fn update(&mut self, event: Msg) {
/// #     }
/// # }
/// #
/// # impl WidgetTest for Win {
/// #     type Widgets = Win;
/// #
/// #     type Streams = ();
/// #
/// #     fn get_widgets(&self) -> Self::Widgets {
/// #         self.clone()
/// #     }
/// #
/// #     fn get_streams(&self) -> Self::Streams {}
/// # }
/// #
/// # impl Widget for Win {
/// #     type Root = Window;
/// #
/// #     fn root(&self) -> Self::Root {
/// #         self.window.clone()
/// #     }
/// #
/// #     fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
/// #         let window = Window::new(WindowType::Toplevel);
/// #
/// #         Win {
/// #             window: window,
/// #         }
/// #     }
/// # }
/// #
/// # #[derive(Msg)]
/// # enum Msg {}
/// # fn main() {
/// let (component, _, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
/// # }
/// ```
pub fn init_test<WIDGET>(model_param: WIDGET::ModelParam) ->
    Result<InitTestComponents<WIDGET>, glib::BoolError>
    where WIDGET: Widget + WidgetTest + 'static,
          WIDGET::Msg: DisplayVariant + 'static,
{
    gtk::init()?;
    let main_context = glib::MainContext::default();
    let _context = main_context.acquire()?;
    let component = create_widget_test::<WIDGET>(model_param);
    Ok(component)
}

/// Initialize a widget.
pub fn init<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, glib::BoolError>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static
{
    let (component, widget, relm) = create_widget::<WIDGET>(model_param);
    init_component::<WIDGET>(component.owned_stream(), widget, &relm);
    Ok(component)
}

/// Create the specified relm `Widget` and run the main event loops.
///
/// ```
/// # use gtk::{Window, WindowType};
/// # use relm::{Relm, Update, Widget};
/// # use relm_derive::Msg;
/// #
/// # struct Win {
/// #     window: Window,
/// # }
/// #
/// # impl Update for Win {
/// #     type Model = ();
/// #     type ModelParam = ();
/// #     type Msg = Msg;
/// #
/// #     fn model(_: &Relm<Self>, _: ()) -> () {
/// #         ()
/// #     }
/// #
/// #     fn update(&mut self, event: Msg) {
/// #     }
/// # }
/// #
/// # impl Widget for Win {
/// #     type Root = Window;
/// #
/// #     fn root(&self) -> Self::Root {
/// #         self.window.clone()
/// #     }
/// #
/// #     fn view(relm: &Relm<Self>, _model: Self::Model) -> Self {
/// #         let window = Window::new(WindowType::Toplevel);
/// #
/// #         Win {
/// #             window: window,
/// #         }
/// #     }
/// # }
/// # #[derive(Msg)]
/// # enum Msg {}
/// # fn main() {
/// # }
/// #
/// # fn run() {
/// /// `Win` is a relm `Widget`.
/// Win::run(()).expect("Win::run failed");
/// # }
/// ```
pub fn run<WIDGET>(model_param: WIDGET::ModelParam) -> Result<(), glib::BoolError>
    where WIDGET: Widget + 'static,
{
    let main_context = glib::MainContext::default();
    let _context = main_context.acquire()?;
    gtk::init()?;
    let _component = init::<WIDGET>(model_param)?;
    gtk::main();
    Ok(())
}

/// Emit the `msg` every `duration` ms.
pub fn interval<F: Fn() -> MSG + 'static, MSG: 'static>(stream: &StreamHandle<MSG>, duration: u32, constructor: F) {
    let stream = stream.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(duration as u64), move || {
        let msg = constructor();
        stream.emit(msg);
        Continue(true)
    });
}

/// After `duration` ms, emit `msg`.
pub fn timeout<F: Fn() -> MSG + 'static, MSG: 'static>(stream: &StreamHandle<MSG>, duration: u32, constructor: F) {
    let stream = stream.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(duration as u64), move || {
        let msg = constructor();
        stream.emit(msg);
        Continue(false)
    });
}

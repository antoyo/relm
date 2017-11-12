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

//! Asynchronous GUI library based on GTK+ and futures/tokio.
//!
//! This library provides a `Widget` trait that you can use to create asynchronous GUI components.
//! This is the trait you will need to implement for your application.
//! It helps you to implement MVC (Model, View, Controller) in an elegant way.
//!
//! ## Installation
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! gtk = "^0.1.2"
//! relm = "^0.5.0"
//! relm-derive = "^0.1.2"
//! ```
//!
//! More info can be found in the [readme](https://github.com/antoyo/relm#relm).

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
)]

/*
 * FIXME: some relm widgets requires { and } (see the rusic music-player) while other do not.
 * FIXME: should not require to import WidgetExt because it calls show().
 * FIXME: cannot have relm event with tuple as a value like:
 * RelmWidget {
 *     RelmEvent((value1, value2)) => …
 * }
 * TODO: use pub(crate) instead of pub so that we're not bound to make the model and msg structs pub.
 *
 * TODO: alternative to tokio with a trait to get FD and add this FD to glib (perhaps using a
 * GSource).
 * For timers, add a connect_timeout!() macro or something.
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
 * TODO: err if trying to use the SimpleMsg custom derive on stable.
 *
 * TODO: add default type of () for Model in Widget when it is stable.
 * TODO: optionnaly multi-threaded.
 * TODO: convert GTK+ callback to Stream (does not seem worth it, nor convenient since it will
 * still need to use USFC for the callback method).
 *
 * These probably won't be needed anymore when switching to futures-glib (single-threaded model).
 * TODO: use weak pointers to avoid leaking.
 * TODO: should have a free function to delete the stream in connect_recv.
 * TODO: try tk-easyloop in another branch.
 */

extern crate futures_glib;
extern crate glib;
extern crate glib_sys;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate relm_core;
extern crate relm_state;

mod component;
mod container;
mod macros;
mod widget;

use futures_glib::Executor;
#[doc(hidden)]
pub use futures_glib::MainLoop;
#[doc(hidden)]
pub use glib::Cast;
#[doc(hidden)]
pub use glib::object::Downcast;
#[doc(hidden)]
pub use glib::translate::{FromGlibPtrNone, ToGlib, ToGlibPtr};
use glib_sys::GType;
#[doc(hidden)]
pub use gobject_sys::{GParameter, g_object_newv};
use gobject_sys::{GObject, GValue};
use libc::{c_char, c_uint};
#[doc(hidden)]
pub use relm_core::EventStream;
pub use relm_state::{
    DisplayVariant,
    IntoOption,
    IntoPair,
    Relm,
    Update,
    UpdateNew,
    create_executor,
    execute,
};
use relm_state::init_component;

pub use component::Component;
pub use container::{Container, ContainerComponent, ContainerWidget};
pub use widget::Widget;

extern "C" {
    pub fn g_object_new_with_properties(object_type: GType, n_properties: c_uint, names: *mut *const c_char,
                                        values: *mut *const GValue) -> *mut GObject;
}

/// Dummy macro to be used with `#[derive(Widget)]`.
///
/// An example can be found [here](https://github.com/antoyo/relm/blob/master/examples/buttons-derive/src/main.rs#L52).
#[macro_export]
macro_rules! impl_widget {
    ($($tt:tt)*) => {
        ()
    };
}

/// Macro to be used as a stable alternative to the #[widget] attribute.
#[macro_export]
macro_rules! relm_widget {
    ($($tts:tt)*) => {
        mod __relm_gen_private {
            use super::*;

            #[derive(Widget)]
            struct __RelmPrivateWidget {
                widget: impl_widget! {
                    $($tts)*
                }
            }
        }

        use_impl_self_type!($($tts)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! use_impl_self_type {
    (impl Widget for $self_type:ident { $($tts:tt)* }) => {
        pub use self::__relm_gen_private::$self_type;
    };
}

fn create_widget_test<WIDGET>(executor: &Executor, model_param: WIDGET::ModelParam) -> Component<WIDGET>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static,
{
    let (widget, component, relm) = create_widget(executor, model_param);
    init_component::<WIDGET>(widget.stream(), component, executor, &relm);
    widget
}

/// Create a new relm widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_component<CHILDWIDGET, WIDGET>(relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
        -> Component<CHILDWIDGET>
    where CHILDWIDGET: Widget + 'static,
          CHILDWIDGET::Msg: DisplayVariant + 'static,
          WIDGET: Widget,
{
    let (widget, component, child_relm) = create_widget::<CHILDWIDGET>(relm.executor(), model_param);
    init_component::<CHILDWIDGET>(widget.stream(), component, relm.executor(), &child_relm);
    widget
}

/// Create a new relm container widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_container<CHILDWIDGET, WIDGET>(relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
        -> ContainerComponent<CHILDWIDGET>
    where CHILDWIDGET: Container + Widget + 'static,
          CHILDWIDGET::Msg: DisplayVariant + 'static,
          WIDGET: Widget,
{
    let (widget, component, child_relm) = create_widget::<CHILDWIDGET>(relm.executor(), model_param);
    let container = component.container().clone();
    let containers = component.other_containers();
    init_component::<CHILDWIDGET>(widget.stream(), component, relm.executor(), &child_relm);
    ContainerComponent::new(widget, container, containers)
}

/// Create a new relm widget with `model_param` as initialization value.
fn create_widget<WIDGET>(executor: &Executor, model_param: WIDGET::ModelParam)
    -> (Component<WIDGET>, WIDGET, Relm<WIDGET>)
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static,
{
    let stream = EventStream::new();

    let relm = Relm::new(executor.clone(), stream.clone());
    let model = WIDGET::model(&relm, model_param);
    let mut widget = WIDGET::view(&relm, model);
    widget.init_view();

    let root = widget.root().clone();
    (Component::new(stream, root), widget, relm)
}

// TODO: remove this workaround.
fn init_gtk() {
    let mut argc = 0;
    unsafe {
        gtk_sys::gtk_init(&mut argc, std::ptr::null_mut());
        gtk::set_initialized();
    }
}

/// Initialize a widget for a test.
///
/// It is to be used this way:
/// ```
/// # extern crate gtk;
/// # #[macro_use]
/// # extern crate relm;
/// # #[macro_use]
/// # extern crate relm_derive;
/// #
/// # use gtk::{Window, WindowType};
/// # use relm::{Relm, Widget};
/// #
/// # struct Win {
/// #     window: Window,
/// # }
/// #
/// # impl Widget for Win {
/// #     type Model = ();
/// #     type ModelParam = ();
/// #     type Msg = Msg;
/// #     type Root = Window;
/// #
/// #     fn model(_: ()) -> () {
/// #         ()
/// #     }
/// #
/// #     fn root(&self) -> &Self::Root {
/// #         &self.window
/// #     }
/// #
/// #     fn update(&mut self, event: Msg, model: &mut Self::Model) {
/// #     }
/// #
/// #     fn view(relm: &Relm<Self>, _model: &Self::Model) -> Self {
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
/// let component = relm::init_test::<Win>(()).unwrap();
/// let widgets = component.widget();
/// # }
/// ```
pub fn init_test<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static
{
    futures_glib::init();
    init_gtk();

    let executor = create_executor();
    let component = create_widget_test::<WIDGET>(&executor, model_param);
    Ok(component)
}

/// Initialize a widget.
pub fn init<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: DisplayVariant + 'static
{
    futures_glib::init();
    gtk::init().map_err(|_| ())?;

    let executor = create_executor();
    let (widget, component, relm) = create_widget::<WIDGET>(&executor, model_param);
    init_component::<WIDGET>(widget.stream(), component, &executor, &relm);
    Ok(widget)
}

/// Create the specified relm `Widget` and run the main event loops.
/// ```
/// # extern crate gtk;
/// # #[macro_use]
/// # extern crate relm;
/// # #[macro_use]
/// # extern crate relm_derive;
/// #
/// # use gtk::{Window, WindowType};
/// # use relm::{Relm, Widget};
/// #
/// # struct Win {
/// #     window: Window,
/// # }
/// #
/// # impl Widget for Win {
/// #     type Model = ();
/// #     type ModelParam = ();
/// #     type Msg = Msg;
/// #     type Root = Window;
/// #
/// #     fn model(_: ()) -> () {
/// #         ()
/// #     }
/// #
/// #     fn root(&self) -> &Self::Root {
/// #         &self.window
/// #     }
/// #
/// #     fn update(&mut self, event: Msg, model: &mut Self::Model) {
/// #     }
/// #
/// #     fn view(relm: &Relm<Self>, _model: &Self::Model) -> Self {
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
/// Win::run(()).unwrap();
/// # }
/// ```
pub fn run<WIDGET>(model_param: WIDGET::ModelParam) -> Result<(), ()>
    where WIDGET: Widget + 'static,
{
    let _component = init::<WIDGET>(model_param)?;
    gtk::main();
    Ok(())
}

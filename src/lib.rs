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
//! ```toml
//! [dependencies]
//! gtk = "^0.1.2"
//! relm = "^0.5.0"
//! relm-derive = "^0.1.2"
//! ```
//!
//! More info can be found in the [readme](https://github.com/antoyo/relm#relm).

#![cfg_attr(feature = "use_impl_trait", feature(conservative_impl_trait))]
#![warn(missing_docs, trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]

/*
 * TODO: warn in the attribute when an event cycle is found.
 * TODO: add a Deref<Widget> for Component?
 * TODO: look at how Elm works with the <canvas> element.
 * TODO: the widget names should start with __relm_field_.
 *
 * TODO: for the #[widget] attribute allow pattern matching by creating a function update(&mut
 * self, Quit: Msg, model: &mut Model) so that we can separate the update function in multiple
 * functions.
 *
 * TODO: reset widget name counters when creating new widget?
 *
 * TODO: refactor the code.
 *
 * TODO: chat client/server example.
 *
 * TODO: err if trying to use the SimpleMsg custom derive on stable.
 *
 * TODO: add Cargo travis/appveyor badges.
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

extern crate futures;
extern crate futures_glib;
extern crate glib;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate log;
extern crate relm_core;

mod component;
mod container;
mod macros;
mod stream;
mod widget;

use std::time::SystemTime;

use futures::{Future, Stream};
use futures::future::Spawn;
use futures_glib::MainContext;
#[doc(hidden)]
pub use glib::Cast;
#[doc(hidden)]
pub use glib::object::Downcast;
#[doc(hidden)]
pub use glib::translate::{FromGlibPtrNone, ToGlib};
#[doc(hidden)]
pub use gobject_sys::g_object_new;
#[doc(hidden)]
pub use relm_core::EventStream;

pub use container::{Container, ContainerWidget, RelmContainer};
pub use component::Component;
use stream::ToStream;
pub use widget::Widget;

/// Dummy macro to be used with `#[derive(Widget)]`.
///
/// An example can be found [here](https://github.com/antoyo/relm/blob/master/examples/buttons-derive/src/main.rs#L52).
#[macro_export]
macro_rules! impl_widget {
    ($($tt:tt)*) => {
        ()
    };
}

macro_rules! relm_connect {
    ($_self:expr, $to_stream:expr, $success_callback:expr, $failure_callback:expr) => {{
        let event_stream = $_self.stream.clone();
        let fail_event_stream = $_self.stream.clone();
        let stream = $to_stream.to_stream();
        stream.map_err(move |error| {
            fail_event_stream.emit($failure_callback(error));
            ()
        })
            .for_each(move |result| {
                event_stream.emit($success_callback(result));
                Ok::<(), STREAM::Error>(())
            }
            .map_err(|_| ()))
    }};
}

macro_rules! relm_connect_ignore {
    ($_self:expr, $to_stream:expr, $success_callback:expr) => {{
        let event_stream = $_self.stream.clone();
        let stream = $to_stream.to_stream();
        stream.map_err(|_| ())
            .for_each(move |result| {
                event_stream.emit($success_callback(result));
                Ok::<(), STREAM::Error>(())
            }
            .map_err(|_| ()))
    }};
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

/// Handle connection of futures to send messages to the [`update()`](trait.Widget.html#tymethod.update) method.
pub struct Relm<WIDGET: Widget> {
    cx: MainContext,
    stream: EventStream<WIDGET::Msg>,
}

impl<WIDGET: Widget> Clone for Relm<WIDGET> {
    fn clone(&self) -> Self {
        Relm {
            cx: self.cx.clone(),
            stream: self.stream.clone(),
        }
    }
}

impl<WIDGET: Widget> Relm<WIDGET> {
    #[cfg(feature = "use_impl_trait")]
    pub fn connect<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM,
            success_callback: CALLBACK, failure_callback: FAILCALLBACK) -> impl Future<Item=(), Error=()>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        relm_connect!(self, to_stream, success_callback, failure_callback)
    }

    #[cfg(not(feature = "use_impl_trait"))]
    /// Connect a `Future` or a `Stream` called `to_stream` to send the message `success_callback`
    /// in case of success and `failure_callback` in case of failure.
    ///
    /// ## Note
    /// This function does not spawn the future.
    /// You'll usually want to use [`Relm::connect_exec()`](struct.Relm.html#method.connect_exec) to both connect and spawn the future.
    pub fn connect<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK,
            failure_callback: FAILCALLBACK) -> Box<Future<Item=(), Error=()>>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        let future = relm_connect!(self, to_stream, success_callback, failure_callback);
        Box::new(future)
    }

    #[cfg(feature = "use_impl_trait")]
    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK)
            -> impl Future<Item=(), Error=()>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        relm_connect_ignore!(self, to_stream, success_callback)
    }

    #[cfg(not(feature = "use_impl_trait"))]
    /// This function is the same as [`Relm::connect()`](struct.Relm.html#method.connect) except it does not take a
    /// `failure_callback`; hence, it ignores the errors.
    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK) ->
        Box<Future<Item=(), Error=()>>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        Box::new(relm_connect_ignore!(self, to_stream, success_callback))
    }

    /// Connect the future `to_stream` and spawn it on the tokio main loop.
    pub fn connect_exec<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK,
            failure_callback: FAILCALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect(to_stream, callback, failure_callback));
    }

    /// Connect the future `to_stream` and spawn it on the tokio main loop, ignoring any error.
    pub fn connect_exec_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect_ignore_err(to_stream, callback));
    }

    /// Spawn a future in the tokio event loop.
    pub fn exec<FUTURE: Future<Item=(), Error=()> + 'static>(&self, future: FUTURE) {
        self.cx.spawn(future);
    }

    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<WIDGET::Msg> {
        &self.stream
    }
}

fn create_widget_test<WIDGET>(cx: &MainContext, model_param: WIDGET::ModelParam) -> Component<WIDGET>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    let (component, relm) = create_widget(cx, model_param);
    init_component::<WIDGET>(&component, cx, &relm);
    component
}

/// Create a new relm widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_component<CHILDWIDGET, WIDGET>(relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
        -> Component<CHILDWIDGET>
    where CHILDWIDGET: Widget + 'static,
          CHILDWIDGET::Msg: Clone + DisplayVariant + 'static,
          WIDGET: Widget,
{
    let (component, child_relm) = create_widget::<CHILDWIDGET>(&relm.cx, model_param);
    init_component::<CHILDWIDGET>(&component, &relm.cx, &child_relm);
    component
}

fn create_widget<WIDGET>(cx: &MainContext, model_param: WIDGET::ModelParam) -> (Component<WIDGET>, Relm<WIDGET>)
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    let stream = EventStream::new();

    let relm = Relm {
        cx: cx.clone(),
        stream: stream.clone(),
    };
    let model = WIDGET::model(&relm, model_param);
    let widget = WIDGET::view(&relm, model);
    widget.borrow_mut().init_view();

    (Component::new(stream, widget), relm)
}

fn init_component<WIDGET>(component: &Component<WIDGET>, cx: &MainContext, relm: &Relm<WIDGET>)
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    let stream = component.stream().clone();
    WIDGET::subscriptions(relm);
    let widget = component.widget_rc().clone();
    let event_future = stream.for_each(move |event| {
        let mut widget = widget.borrow_mut();
        update_widget(&mut *widget, event.clone());
        Ok(())
    });
    cx.spawn(event_future);
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
/// #     fn view(relm: Relm<Msg>, _model: &Self::Model) -> Self {
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
/// let component = relm::init_test::<Win>().unwrap();
/// let widgets = component.widget();
/// # }
/// ```
pub fn init_test<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static
{
    futures_glib::init();
    init_gtk();

    let cx = MainContext::default(|cx| cx.clone());
    let component = create_widget_test::<WIDGET>(&cx);
    Ok(component)
}

fn init<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static
{
    futures_glib::init();
    gtk::init()?;

    let cx = MainContext::default(|cx| cx.clone());
    let (component, relm) = create_widget::<WIDGET>(&cx, model_param);
    init_component::<WIDGET>(&component, &cx, &relm);
    Ok(component)
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
/// #     fn view(relm: Relm<Msg>, _model: &Self::Model) -> Self {
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
          WIDGET::ModelParam: Default,
{
    let _component = init::<WIDGET>(model_param)?;
    gtk::main();
    Ok(())
}

fn update_widget<WIDGET>(widget: &mut WIDGET, event: WIDGET::Msg)
    where WIDGET: Widget,
{
    if cfg!(debug_assertions) {
        let time = SystemTime::now();
        let debug = event.display_variant();
        let debug =
            if debug.len() > 100 {
                format!("{}…", &debug[..100])
            }
            else {
                debug.to_string()
            };
        widget.update(event);
        if let Ok(duration) = time.elapsed() {
            let ms = duration.subsec_nanos() as u64 / 1_000_000 + duration.as_secs() * 1000;
            if ms >= 200 {
                warn!("The update function was slow to execute for message {}: {}ms", debug, ms);
            }
        }
    }
    else {
        widget.update(event)
    }
}

/// Format trait for enum variants.
///
/// `DisplayVariant` is similar to `Debug`, but only works on enum and does not list the
/// variants' parameters.
///
/// This is used internally by the library.
pub trait DisplayVariant {
    /// Formats the current variant of the enum.
    fn display_variant(&self) -> &'static str;
}

impl DisplayVariant for () {
    fn display_variant(&self) -> &'static str {
        ""
    }
}

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
 * TODO: look at how Elm works with the <canvas> element.
 * TODO: support msg variant with multiple values?
   TODO: after switching to futures-glib, remove the unnecessary Arc, Mutex and Clone.
 * FIXME: the widget-list example can trigger (and is broken) the following after removing widgets, adding new
 * widgets again and using these new widgets:
 * GLib-CRITICAL **: g_io_channel_read_unichar: assertion 'channel->is_readable' failed
 *
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
 * TODO: try to avoid having two update() functions by adding the futures to a struct that's
 * returned from the update() function and these futures will then be added to the tokio loop (that
 * probably requires boxing the futures, which we want to avoid).
 * TODO: should have a free function to delete the stream in connect_recv.
 * TODO: automatically create the update_command() function from the events present in the view
 * (or by splitting the update() fucntion).
 * TODO: try tk-easyloop in another branch.
 */

extern crate futures;
extern crate glib;
extern crate glib_itc;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate log;
extern crate relm_core;

mod component;
mod container;
pub mod gtk_ext;
mod macros;
mod stream;
mod widget;

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use futures::{Future, Stream};
use glib::Continue;
#[doc(hidden)]
pub use glib::Cast;
#[doc(hidden)]
pub use glib::object::Downcast;
#[doc(hidden)]
pub use glib::translate::{FromGlibPtrNone, ToGlib};
use glib_itc::{Receiver, channel};
#[doc(hidden)]
pub use gobject_sys::g_object_new;
use relm_core::Core;
#[doc(hidden)]
pub use relm_core::{EventStream, Handle, Remote};

use component::Comp;
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
        pub use __relm_gen_private::$self_type;
    };
}

// TODO: remove this hack.
/// A small type to avoid running the destructor of `T`
pub struct ManuallyDrop<T> { inner: Option<T> }

impl<T> ManuallyDrop<T> {
    pub fn new(t: T) -> ManuallyDrop<T> {
        ManuallyDrop { inner: Some(t) }
    }
}

impl<T> Deref for ManuallyDrop<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<T> Drop for ManuallyDrop<T> {
    fn drop(&mut self) {
        std::mem::forget(self.inner.take())
    }
}

/// Handle connection of futures to send messages to the [`update()`](trait.Widget.html#tymethod.update) and
/// [`update_command()`](trait.Widget.html#method.update_command) methods.
pub struct Relm<MSG: Clone + DisplayVariant> {
    handle: Handle,
    stream: EventStream<MSG>,
}

impl<MSG: Clone + DisplayVariant + Send + 'static> Relm<MSG> {
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
    /// You'll usually want to use [`Relm::connect_exec()`](struct.Relm.html#method.connect_exec) to both connect and
    /// spawn the future.
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
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
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK) ->
            Box<Future<Item=(), Error=()>>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        Box::new(relm_connect_ignore!(self, to_stream, success_callback))
    }

    /// Connect the future `to_stream` and spawn it on the tokio main loop.
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
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
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
    pub fn connect_exec_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect_ignore_err(to_stream, callback));
    }

    /// Spawn a future in the tokio event loop.
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
    pub fn exec<FUTURE: Future<Item=(), Error=()> + 'static>(&self, future: FUTURE) {
        self.handle.spawn(future);
    }

    /// Get a handle to the tokio event loop.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

/// Handle to the tokio event loop, to be used from the GTK+ thread.
#[derive(Clone)]
pub struct RemoteRelm<WIDGET: Widget> {
    model: Arc<Mutex<WIDGET::Model>>,
    remote: Remote,
    stream: EventStream<WIDGET::Msg>,
}

impl<WIDGET: Widget> RemoteRelm<WIDGET> {
    /// Get the shared model.
    pub fn model(&self) -> &Arc<Mutex<WIDGET::Model>> {
        &self.model
    }

    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<WIDGET::Msg> {
        &self.stream
    }
}

fn create_widget_test<WIDGET>(remote: &Remote, model_param: WIDGET::ModelParam) -> Component<WIDGET>
    where WIDGET: Widget + Clone + 'static,
          WIDGET::Model: Clone + Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static,
{
    let component = create_widget(remote, model_param);
    init_component::<WIDGET>(&component, remote);
    Component::new(component)
}

/// Create a new relm widget without adding it to an existing widget.
/// This is useful when a relm widget is at the root of another relm widget.
pub fn create_component<CHILDWIDGET, WIDGET>(relm: &RemoteRelm<WIDGET>,
        model_param: CHILDWIDGET::ModelParam) -> Component<CHILDWIDGET>
    where CHILDWIDGET: Widget + 'static,
          CHILDWIDGET::Model: Clone + Send,
          CHILDWIDGET::Msg: Clone + DisplayVariant + Send + 'static,
          WIDGET: Widget,
{
    let component = create_widget::<CHILDWIDGET>(&relm.remote, model_param);
    init_component::<CHILDWIDGET>(&component, &relm.remote);
    Component::new(component)
}

fn create_widget<WIDGET>(remote: &Remote, model_param: WIDGET::ModelParam) -> Comp<WIDGET>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    let (sender, mut receiver) = channel();
    let stream = EventStream::new(Arc::new(Mutex::new(sender)));

    let (widget, model) = {
        let model = Arc::new(Mutex::new(WIDGET::model(model_param)));
        let relm = RemoteRelm {
            model: model,
            remote: remote.clone(),
            stream: stream.clone(),
        };
        let view = {
            let model_guard = relm.model.lock().unwrap();
            WIDGET::view(&relm, &*model_guard)
        };
        (view, relm.model)
    };
    widget.init_view();

    {
        let mut widget = widget.clone();
        let stream = stream.clone();
        let model = model.clone();
        receiver.connect_recv(move || {
            if let Some(event) = stream.pop_ui_events() {
                let mut model = model.lock().unwrap();
                update_widget(&mut widget, event, &mut *model);
            }
            Continue(true)
        });
    }

    Comp {
        model: model,
        _receiver: Arc::new(receiver),
        stream: stream,
        widget: widget,
    }
}

fn init_component<WIDGET>(component: &Comp<WIDGET>, remote: &Remote)
    where WIDGET: Widget + 'static,
          WIDGET::Model: Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static,
{
    let stream = component.stream.clone();
    let model = component.model.clone();
    remote.spawn(move |handle| {
        let relm = Relm {
            handle: handle.clone(),
            stream: stream.clone(),
        };
        WIDGET::subscriptions(&relm);
        let event_future = stream.for_each(move |event| {
            let mut model = model.lock().unwrap();
            WIDGET::update_command(&relm, event, &mut *model);
            Ok(())
        });
        handle.spawn(event_future);
        Ok(())
    });
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
/// # use relm::{RemoteRelm, Widget};
/// #
/// # #[derive(Clone)]
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
/// #     fn view(relm: &RemoteRelm<Self>, _model: &Self::Model) -> Self {
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
/// # }
/// ```
pub fn init_test<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + Clone + 'static,
          WIDGET::Model: Clone + Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static
{
    init_gtk();

    let remote = Core::run();
    let component = create_widget_test::<WIDGET>(&remote, model_param);
    Ok(component)
}

fn init<WIDGET>(model_param: WIDGET::ModelParam) -> Result<Component<WIDGET>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Model: Clone + Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static
{
    gtk::init()?;

    let remote = Core::run();
    let component = create_widget::<WIDGET>(&remote, model_param);
    init_component::<WIDGET>(&component, &remote);
    Ok(Component::new(component))
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
/// # use relm::{RemoteRelm, Widget};
/// #
/// # #[derive(Clone)]
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
/// #     fn view(relm: &RemoteRelm<Self>, _model: &Self::Model) -> Self {
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
          WIDGET::Model: Clone + Send,
          WIDGET::ModelParam: Default,
          WIDGET::Msg: Send,
{
    let _component = init::<WIDGET>(model_param)?;
    gtk::main();
    Ok(())
}

fn update_widget<WIDGET>(widget: &mut WIDGET, event: WIDGET::Msg, model: &mut WIDGET::Model)
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
        widget.update(event, model);
        if let Ok(duration) = time.elapsed() {
            let ms = duration.subsec_nanos() as u64 / 1_000_000 + duration.as_secs() * 1000;
            if ms >= 200 {
                warn!("The update function was slow to execute for message {}: {}ms", debug, ms);
            }
        }
    }
    else {
        widget.update(event, model)
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

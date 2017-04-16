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
//! relm = "^0.4.0"
//! relm-derive = "^0.1.2"
//! ```
//!
//! More info can be found in the [readme](https://github.com/antoyo/relm#relm).

#![cfg_attr(feature = "use_impl_trait", feature(conservative_impl_trait))]
#![warn(missing_docs, trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces, unused_qualifications, unused_results)]

/*
 * FIXME: the widget-list example can trigger the following after removing widgets, adding new
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
 * TODO: automatically create the update_command() function from the events present in the view (or by splitting the update() fucntion).
 * TODO: try tk-easyloop in another branch.
 */

extern crate cairo;
extern crate futures;
extern crate glib;
extern crate glib_itc;
extern crate gobject_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate log;
extern crate relm_core;

pub mod gtk_ext;
mod macros;
mod stream;
mod widget;

use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use futures::{Future, Stream};
use glib::Continue;
#[doc(hidden)]
pub use glib::object::Downcast;
#[doc(hidden)]
pub use glib::translate::{FromGlibPtrNone, ToGlib};
use glib_itc::{Receiver, channel};
#[doc(hidden)]
pub use gobject_sys::g_object_new;
use gtk::{ContainerExt, IsA, Object, WidgetExt};
use relm_core::Core;
#[doc(hidden)]
pub use relm_core::{EventStream, Handle, Remote};

use self::stream::ToStream;
pub use self::widget::*;

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
    (impl Widget<$msg_type:ty> for $self_type:ident { $($tts:tt)* }) => {
        pub use __relm_gen_private::$self_type;
    };
}

/// Widget that was added by the `ContainerWidget::add_widget()` method.
///
/// ## Warning
/// You must keep your components as long as you want them to send/receive events.
/// Common practice is to store `Component`s in the `Widget` struct (see the [communication
/// example](https://github.com/antoyo/relm/blob/master/examples/communication.rs#L182-L188)).
/// The `#[widget]` attribute takes care of storing them in the struct automatically (see the
/// [communication-attribute example](https://github.com/antoyo/relm/blob/master/examples/communication-attribute.rs)).
#[must_use]
#[derive(Clone)]
pub struct Component<MODEL, MSG: Clone + DisplayVariant, WIDGET> {
    model: Arc<Mutex<MODEL>>,
    _receiver: Arc<Receiver>,
    stream: EventStream<MSG>,
    widget: WIDGET,
}

impl<MODEL, MSG: Clone + DisplayVariant, WIDGET> Component<MODEL, MSG, WIDGET> {
    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

impl<MODEL, MSG: Clone + DisplayVariant, WIDGET> Drop for Component<MODEL, MSG, WIDGET> {
    fn drop(&mut self) {
        let _ = self.stream.close();
    }
}

impl<MODEL, MSG: Clone + DisplayVariant, WIDGET: ContainerExt> ContainerExt for Component<MODEL, MSG, WIDGET> {
    fn add<T: IsA<gtk::Widget>>(&self, widget: &T) { self.widget.add(widget) }
    fn check_resize(&self) { self.widget.check_resize() }
    fn child_notify<T: IsA<gtk::Widget>>(&self, child: &T, child_property: &str) { self.widget.child_notify(child, child_property) }
    fn child_type(&self) -> gtk::Type { self.widget.child_type() }
    fn get_border_width(&self) -> u32 { self.widget.get_border_width() }
    fn get_children(&self) -> Vec<gtk::Widget> { self.widget.get_children() }
    fn get_focus_child(&self) -> Option<gtk::Widget> { self.widget.get_focus_child() }
    fn get_focus_hadjustment(&self) -> Option<gtk::Adjustment> { self.widget.get_focus_hadjustment() }
    fn get_focus_vadjustment(&self) -> Option<gtk::Adjustment> { self.widget.get_focus_vadjustment() }
    fn get_resize_mode(&self) -> gtk::ResizeMode { self.widget.get_resize_mode() }
    fn propagate_draw<T: IsA<gtk::Widget>>(&self, child: &T, cr: &cairo::Context) { self.widget.propagate_draw(child, cr) }
    fn remove<T: IsA<gtk::Widget>>(&self, widget: &T) { self.widget.remove(widget) }
    fn resize_children(&self) { self.widget.resize_children() }
    fn set_border_width(&self, border_width: u32) { self.widget.set_border_width(border_width) }
    fn set_focus_chain(&self, focusable_widgets: &[gtk::Widget]) { self.widget.set_focus_chain(focusable_widgets) }
    fn set_focus_child<T: IsA<gtk::Widget>>(&self, child: Option<&T>) { self.widget.set_focus_child(child) }
    fn set_focus_hadjustment(&self, adjustment: &gtk::Adjustment) { self.widget.set_focus_hadjustment(adjustment) }
    fn set_focus_vadjustment(&self, adjustment: &gtk::Adjustment) { self.widget.set_focus_vadjustment(adjustment) }
    fn set_reallocate_redraws(&self, needs_redraws: bool) { self.widget.set_reallocate_redraws(needs_redraws) }
    fn set_resize_mode(&self, resize_mode: gtk::ResizeMode) { self.widget.set_resize_mode(resize_mode) }
    fn unset_focus_chain(&self) { self.widget.unset_focus_chain() }
    fn set_property_child(&self, child: Option<&gtk::Widget>) { self.widget.set_property_child(child) }
    fn connect_add<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_check_resize<F: Fn(&Self) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_remove<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_set_focus_child<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
}

/// Handle connection of futures to send messages to the [`update()`](trait.Widget.html#tymethod.update) and [`update_command()`](trait.Widget.html#method.update_command) methods.
pub struct Relm<MSG: Clone + DisplayVariant> {
    handle: Handle,
    stream: EventStream<MSG>,
}

impl<MSG: Clone + DisplayVariant + Send + 'static> Relm<MSG> {
    #[cfg(feature = "use_impl_trait")]
    pub fn connect<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK, failure_callback: FAILCALLBACK) -> impl Future<Item=(), Error=()>
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
    ///
    /// ## Warning
    /// This function **must** be executed of the tokio thread, i.e. in the
    /// [`subscriptions()`](trait.Widget.html#method.subscriptions) or
    /// [`update_command()`](trait.Widget.html#method.update_command) methods.
    pub fn connect<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK, failure_callback: FAILCALLBACK) -> Box<Future<Item=(), Error=()>>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        let future = relm_connect!(self, to_stream, success_callback, failure_callback);
        Box::new(future)
    }

    #[cfg(feature = "use_impl_trait")]
    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK) -> impl Future<Item=(), Error=()>
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
    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK) -> Box<Future<Item=(), Error=()>>
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
    pub fn connect_exec<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK, failure_callback: FAILCALLBACK)
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
pub struct RemoteRelm<MSG: Clone + DisplayVariant> {
    remote: Remote,
    stream: EventStream<MSG>,
}

impl<'a, MSG: Clone + DisplayVariant> RemoteRelm<MSG> {
    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

fn create_widget_test<WIDGET>(remote: &Remote) -> (Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>, WIDGET)
    where WIDGET: Widget + Clone + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    // TODO: remove redundancy with create_widget.
    let (sender, mut receiver) = channel();
    let stream = EventStream::new(Arc::new(sender));

    let (widget, model) = {
        let relm = RemoteRelm {
            remote: remote.clone(),
            stream: stream.clone(),
        };
        let model = WIDGET::model();
        (WIDGET::view(relm, &model), model)
    };

    let container = widget.container().clone();
    let model = Arc::new(Mutex::new(model));

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

    let component = Component {
        model: model,
        _receiver: Arc::new(receiver),
        stream: stream,
        widget: container,
    };

    (component, widget)
}

fn create_widget<WIDGET>(remote: &Remote) -> Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>
    where WIDGET: Widget + 'static,
          WIDGET::Msg: Clone + DisplayVariant + 'static,
{
    let (sender, mut receiver) = channel();
    let stream = EventStream::new(Arc::new(sender));

    let (mut widget, model) = {
        let relm = RemoteRelm {
            remote: remote.clone(),
            stream: stream.clone(),
        };
        let model = WIDGET::model();
        (WIDGET::view(relm, &model), model)
    };

    let container = widget.container().clone();
    let model = Arc::new(Mutex::new(model));

    {
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

    Component {
        model: model,
        _receiver: Arc::new(receiver),
        stream: stream,
        widget: container,
    }
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
/// #     type Container = Window;
/// #     type Model = ();
/// #     type Msg = Msg;
/// #
/// #     fn container(&self) -> &Self::Container {
/// #         &self.window
/// #     }
/// #
/// #     fn model() -> () {
/// #         ()
/// #     }
/// #
/// #     fn update(&mut self, event: Msg, model: &mut Self::Model) {
/// #     }
/// #
/// #     fn view(relm: RemoteRelm<Msg>, _model: &Self::Model) -> Self {
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
/// let (_component, widgets) = relm::init_test::<Win>().unwrap();
/// # }
/// ```
///
/// ## Warning
/// You **should** use `_component` instead of `_` to avoid dropping it too early, which will cause
/// events to not be sent.
pub fn init_test<WIDGET>() -> Result<(Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>, WIDGET), ()>
    where WIDGET: Widget + Clone + 'static,
          WIDGET::Model: Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static
{
    init_gtk();

    let remote = Core::run();
    let (component, widgets) = create_widget_test::<WIDGET>(&remote);
    init_component::<WIDGET>(&component, &remote);
    Ok((component, widgets))
}

fn init<WIDGET>() -> Result<Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>, ()>
    where WIDGET: Widget + 'static,
          WIDGET::Model: Send,
          WIDGET::Msg: Clone + DisplayVariant + Send + 'static
{
    gtk::init()?;

    let remote = Core::run();
    let component = create_widget::<WIDGET>(&remote);
    init_component::<WIDGET>(&component, &remote);
    Ok(component)
}

fn init_component<WIDGET>(component: &Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>, remote: &Remote)
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
/// #     type Container = Window;
/// #     type Model = ();
/// #     type Msg = Msg;
/// #
/// #     fn container(&self) -> &Self::Container {
/// #         &self.window
/// #     }
/// #
/// #     fn model() -> () {
/// #         ()
/// #     }
/// #
/// #     fn update(&mut self, event: Msg, model: &mut Self::Model) {
/// #     }
/// #
/// #     fn view(relm: RemoteRelm<Msg>, _model: &Self::Model) -> Self {
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
/// relm::run::<Win>();
/// # }
/// ```
pub fn run<WIDGET>() -> Result<(), ()>
    where WIDGET: Widget + 'static,
          WIDGET::Model: Send,
          WIDGET::Msg: Send,
{
    let _component = init::<WIDGET>()?;
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

/// Extension trait for GTK+ containers to add and remove relm `Widget`s.
pub trait ContainerWidget
    where Self: ContainerExt
{
    /// Add a relm `Widget` to the current GTK+ container.
    ///
    /// # Note
    ///
    /// The returned `Component` must be stored in a `Widget`. If it is not stored, a communication
    /// receiver will be droped which will cause events to be ignored for this widget.
    fn add_widget<WIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<WIDGET::Model, WIDGET::Msg, WIDGET::Container>
        where MSG: Clone + DisplayVariant + Send + 'static,
              WIDGET: Widget + 'static,
              WIDGET::Container: IsA<Object> + WidgetExt,
              WIDGET::Model: Send,
              WIDGET::Msg: Clone + DisplayVariant + Send + 'static,
    {
        let component = create_widget::<WIDGET>(&relm.remote);
        self.add(&component.widget);
        component.widget.show_all();
        init_component::<WIDGET>(&component, &relm.remote);
        component
    }

    /// Remove a relm `Widget` from the current GTK+ container.
    fn remove_widget<MODEL, MSG: Clone + DisplayVariant + 'static, WIDGET>(&self, component: Component<MODEL, MSG, WIDGET>)
        where WIDGET: IsA<gtk::Widget>,
    {
        self.remove(&component.widget);
    }
}

impl<W: ContainerExt> ContainerWidget for W { }

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

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

/*
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

#![feature(conservative_impl_trait)]

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

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use futures::{Future, Stream};
use glib::Continue;
pub use glib::object::Downcast;
pub use glib::translate::{FromGlibPtrNone, ToGlib};
use glib_itc::{Receiver, channel};
pub use gobject_sys::{GObject, g_object_new};
use gtk::{ContainerExt, IsA, Object, WidgetExt};
use relm_core::Core;
pub use relm_core::{EventStream, Handle, Remote};

pub use self::Error::*;
use self::stream::ToStream;
pub use self::widget::*;

#[must_use]
#[derive(Clone)]
pub struct Component<MODEL, MSG: Clone + DisplayVariant, WIDGET> {
    model: Arc<Mutex<MODEL>>,
    _receiver: Arc<Receiver>,
    stream: EventStream<MSG>,
    widget: WIDGET,
}

impl<MODEL, MSG: Clone + DisplayVariant, WIDGET> Component<MODEL, MSG, WIDGET> {
    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

impl<MODEL, MSG: Clone + DisplayVariant, WIDGET> Drop for Component<MODEL, MSG, WIDGET> {
    fn drop(&mut self) {
        self.stream.close().ok();
    }
}

#[derive(Debug)]
pub enum Error {
    GtkInit,
    Io(io::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            GtkInit => write!(formatter, "Cannot init GTK+"),
            Io(ref error) => write!(formatter, "IO error: {}", error),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            GtkInit => "Cannot init GTK+",
            Io(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            GtkInit => None,
            Io(ref error) => Some(error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Io(error)
    }
}

impl From<()> for Error {
    fn from((): ()) -> Error {
        GtkInit
    }
}

pub struct Relm<MSG: Clone + DisplayVariant> {
    handle: Handle,
    stream: EventStream<MSG>,
}

impl<MSG: Clone + DisplayVariant + Send + 'static> Relm<MSG> {
    pub fn connect<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK, failure_callback: FAILCALLBACK) -> impl Future<Item=(), Error=()>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        let event_stream = self.stream.clone();
        let fail_event_stream = self.stream.clone();
        let stream = to_stream.to_stream();
        stream.map_err(move |error| {
            fail_event_stream.emit(failure_callback(error));
            ()
        })
            .for_each(move |result| {
                event_stream.emit(success_callback(result));
                Ok::<(), STREAM::Error>(())
            }
            .map_err(|_| ()))
    }

    pub fn connect_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, success_callback: CALLBACK) -> impl Future<Item=(), Error=()>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        let event_stream = self.stream.clone();
        let stream = to_stream.to_stream();
        stream.map_err(|_| ())
            .for_each(move |result| {
                event_stream.emit(success_callback(result));
                Ok::<(), STREAM::Error>(())
            }
            .map_err(|_| ()))
    }

    pub fn connect_exec<CALLBACK, FAILCALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK, failure_callback: FAILCALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              FAILCALLBACK: Fn(STREAM::Error) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect(to_stream, callback, failure_callback));
    }

    pub fn connect_exec_ignore_err<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect_ignore_err(to_stream, callback));
    }

    pub fn exec<FUTURE: Future<Item=(), Error=()> + 'static>(&self, future: FUTURE) {
        self.handle.spawn(future);
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn run<WIDGET>() -> Result<(), Error>
        where WIDGET: Widget<MSG> + 'static,
              WIDGET::Model: Send,
    {
        let _component = init::<WIDGET, MSG>()?;
        gtk::main();
        Ok(())
    }

    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

pub struct RemoteRelm<MSG: Clone + DisplayVariant> {
    remote: Remote,
    stream: EventStream<MSG>,
}

impl<'a, MSG: Clone + DisplayVariant> RemoteRelm<MSG> {
    pub fn stream(&self) -> &EventStream<MSG> {
        &self.stream
    }
}

fn create_widget_test<WIDGET, MSG>(remote: &Remote) -> (Component<WIDGET::Model, MSG, WIDGET::Container>, WIDGET)
    where WIDGET: Widget<MSG> + Clone + 'static,
          MSG: Clone + DisplayVariant + 'static,
{
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

fn create_widget<WIDGET, MSG>(remote: &Remote) -> Component<WIDGET::Model, MSG, WIDGET::Container>
    where WIDGET: Widget<MSG> + 'static,
          MSG: Clone + DisplayVariant + 'static,
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

pub fn init_test<WIDGET, MSG: Clone + DisplayVariant + Send + 'static>() -> Result<(Component<WIDGET::Model, MSG, WIDGET::Container>, WIDGET), Error>
    where WIDGET: Widget<MSG> + Clone + 'static,
          WIDGET::Model: Send,
{
    init_gtk();

    let remote = Core::run();
    let (component, widgets) = create_widget_test::<WIDGET, MSG>(&remote);
    init_component::<WIDGET, MSG>(&component, &remote);
    Ok((component, widgets))
}

fn init<WIDGET, MSG: Clone + DisplayVariant + Send + 'static>() -> Result<Component<WIDGET::Model, MSG, WIDGET::Container>, Error>
    where WIDGET: Widget<MSG> + 'static,
          WIDGET::Model: Send,
    {
    gtk::init()?;

    let remote = Core::run();
    let component = create_widget::<WIDGET, MSG>(&remote);
    init_component::<WIDGET, MSG>(&component, &remote);
    Ok(component)
}

fn init_component<WIDGET, MSG>(component: &Component<WIDGET::Model, MSG, WIDGET::Container>, remote: &Remote)
    where WIDGET: Widget<MSG> + 'static,
          WIDGET::Model: Send,
          MSG: Clone + DisplayVariant + Send + 'static,
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

fn update_widget<MSG, WIDGET>(widget: &mut WIDGET, event: MSG, model: &mut WIDGET::Model)
    where MSG: Clone + DisplayVariant,
          WIDGET: Widget<MSG>,
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

pub trait ContainerWidget
    where Self: ContainerExt
{
    /**
     * Add a widget `WIDGET` to the current widget.
     *
     * # Note
     *
     * The returned `Component` must be stored in a `Widget`. If it is not stored, a communication
     * receiver will be droped which will cause events to be ignored for this widget.
     */
    fn add_widget<WIDGET, MSG, WIDGETMSG>(&self, relm: &RemoteRelm<MSG>) -> Component<WIDGET::Model, WIDGETMSG, WIDGET::Container>
        where MSG: Clone + DisplayVariant + Send + 'static,
              WIDGET: Widget<WIDGETMSG> + 'static,
              WIDGET::Container: IsA<Object> + WidgetExt,
              WIDGET::Model: Send,
              WIDGETMSG: Clone + DisplayVariant + Send + 'static,
    {
        let component = create_widget::<WIDGET, WIDGETMSG>(&relm.remote);
        self.add(&component.widget);
        component.widget.show_all();
        init_component::<WIDGET, WIDGETMSG>(&component, &relm.remote);
        component
    }

    fn remove_widget<MODEL, MSG: Clone + DisplayVariant + 'static, WIDGET>(&self, component: Component<MODEL, MSG, WIDGET>)
        where WIDGET: IsA<gtk::Widget>,
    {
        self.remove(&component.widget);
    }
}

impl<W: ContainerExt> ContainerWidget for W { }

pub trait DisplayVariant {
    fn display_variant(&self) -> &'static str;
}

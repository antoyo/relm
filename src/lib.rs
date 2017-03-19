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
 * TODO: Use two update functions (one for errors, one for success/normal behavior).
 *
 * TODO: chat client/server example.
 *
 * TODO: try tk-easyloop in another branch.
 *
 * TODO: err if trying to use the SimpleMsg custom derive on stable.
 *
 * TODO: add Cargo travis badge.
 *
 * TODO: add default type of () for Model in Widget when it is stable.
 * TODO: use macros 2.0 instead for the:
 * * view: to create the dependencies between the view items and the model.
 * * model: to add boolean fields in an inner struct specifying which parts of the view to update
 * *        after the update.
 * * update: to set the boolean fields to true depending on which parts of the model was updated.
 * * create default values for gtk widgets (like Label::new(None)).
 * * create attributes for constructor gtk widgets (like orientation for Box::new(orientation)).
 * TODO: optionnaly multi-threaded.
 * TODO: try to avoid having two update() functions by adding the futures to a struct that's
 * returned from the update() function and these futures will then be added to the tokio loop (that
 * probably requires boxing the futures, which we want to avoid).
 * TODO: convert GTK+ callback to Stream (does not seem worth it, nor convenient since it will
 * still need to use USFC for the callback method).
 */

#![feature(conservative_impl_trait)]

extern crate futures;
extern crate glib;
extern crate glib_itc;
extern crate gtk;
#[macro_use]
extern crate log;
extern crate relm_core;

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
use glib_itc::{Receiver, channel};
use gtk::{ContainerExt, IsA, Object, WidgetExt};
use relm_core::Core;
pub use relm_core::{EventStream, Handle, Remote};

pub use self::Error::*;
use self::stream::ToStream;
pub use self::widget::*;

#[must_use]
pub struct Component<MODEL, MSG: Clone + DisplayVariant, WIDGET> {
    model: Arc<Mutex<MODEL>>,
    _receiver: Receiver,
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
    pub fn connect<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK) -> impl Future<Item=(), Error=()>
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        let event_stream = self.stream.clone();
        let stream = to_stream.to_stream();
        stream.map_err(|_| ()).for_each(move |result| {
            event_stream.emit(callback(result));
            Ok::<(), ()>(())
        }
            // TODO: handle errors.
            .map_err(|_| ()))
    }

    pub fn connect_exec<CALLBACK, STREAM, TOSTREAM>(&self, to_stream: TOSTREAM, callback: CALLBACK)
        where CALLBACK: Fn(STREAM::Item) -> MSG + 'static,
              STREAM: Stream + 'static,
              TOSTREAM: ToStream<STREAM, Item=STREAM::Item, Error=STREAM::Error> + 'static,
    {
        self.exec(self.connect(to_stream, callback));
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
        gtk::init()?;

        let remote = Core::run();
        let component = create_widget::<WIDGET, MSG>(&remote);
        init_component::<WIDGET, MSG>(&component, &remote);
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
        WIDGET::new(relm)
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
            // TODO: should have a free function to delete the stream.
            Continue(true)
        });
    }

    Component {
        model: model,
        _receiver: receiver,
        stream: stream,
        widget: container,
    }
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

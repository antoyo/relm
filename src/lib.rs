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
 * TODO: remove_widget() should remove the stream.
 * TODO: integrate the tokio main loop into the GTK+ main loop so that the example nested-loop
 * works:
 *  * use another method to block a callback than gtk::main().
 *  * * A kind of lock which is unlocked when update() finish.
 *  * Running gtk::main() in update() block Core::turn() because it creates an infinite loop.
 *  * Find a solution for synchronous callbacks.
 * TODO: chat client/server example.
 *
 * TODO: try tk-easyloop in another branch.
 *
 * TODO: err if trying to use the SimpleMsg custom derive on stable.
 * TODO: Use two update functions (one for errors, one for success/normal behavior).
 *
 * TODO: add Cargo travis badge.
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
pub use relm_core::{EventStream, Handle};

pub use self::Error::*;
use self::stream::ToStream;
pub use self::widget::*;

pub struct Component<D, M: Clone + DisplayVariant, W> {
    model: Arc<Mutex<D>>,
    receiver: Receiver,
    stream: EventStream<M>,
    widget: W,
}

impl<D, M: Clone + DisplayVariant, W> Component<D, M, W> {
    pub fn stream(&self) -> &EventStream<M> {
        &self.stream
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

pub struct Relm<M: Clone + DisplayVariant> {
    handle: Handle,
    stream: EventStream<M>,
}

impl<M: Clone + DisplayVariant + Send + 'static> Relm<M> {
    pub fn connect<C, S, T>(&self, to_stream: T, callback: C) -> impl Future<Item=(), Error=()>
        where C: Fn(S::Item) -> M + 'static,
              S: Stream + 'static,
              T: ToStream<S, Item=S::Item, Error=S::Error> + 'static,
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

    pub fn connect_exec<C, S, T>(&self, to_stream: T, callback: C)
        where C: Fn(S::Item) -> M + 'static,
              S: Stream + 'static,
              T: ToStream<S, Item=S::Item, Error=S::Error> + 'static,
    {
        self.exec(self.connect(to_stream, callback));
    }

    pub fn exec<F: Future<Item=(), Error=()> + 'static>(&self, future: F) {
        self.handle.spawn(future);
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn run<D>() -> Result<(), Error>
        where D: Widget<M> + 'static,
              D::Model: Send,
    {
        gtk::init()?;

        let component = create_widget::<D, M>();
        let stream = component.stream;
        let model = component.model;
        Core::run(move |handle| {
            let relm = Relm {
                handle: handle.clone(),
                stream: stream.clone(),
            };
            D::subscriptions(&relm);
            let event_future = stream.for_each(move |event| {
                let mut model = model.lock().unwrap();
                D::update_command(&relm, event, &mut *model);
                Ok(())
            });
            handle.spawn(event_future);
            Ok(())
        });
        Ok(())
    }

    // TODO: delete this method when the connect macros are no longer required.
    pub fn stream(&self) -> &EventStream<M> {
        &self.stream
    }
}

fn create_widget<D, M>() -> Component<D::Model, M, D::Container>
    where D: Widget<M> + 'static,
          M: Clone + DisplayVariant + 'static,
{
    let (sender, mut receiver) = channel();
    let stream = EventStream::new(Arc::new(sender));

    let (mut widget, model) = D::new(&stream);

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
        receiver: receiver,
        stream: stream,
        widget: container,
    }
}

fn update_widget<M, W>(widget: &mut W, event: M, model: &mut W::Model)
    where M: Clone + DisplayVariant,
          W: Widget<M>,
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
    fn add_widget<D, M>(&self) -> Component<D::Model, M, D::Container>
        where D: Widget<M> + 'static,
              D::Container: IsA<Object> + WidgetExt,
              M: Clone + DisplayVariant + 'static,
    {
        let component = create_widget::<D, M>();
        self.add(&component.widget);
        component.widget.show_all();
        component
    }

    // TODO: remove the EventStream also.
    fn remove_widget<D, M: Clone + DisplayVariant, W>(&self, widget: Component<D, M, W>)
        where W: IsA<gtk::Widget>,
    {
        self.remove(&widget.widget);
    }
}

impl<W: ContainerExt> ContainerWidget for W { }

pub trait DisplayVariant {
    fn display_variant(&self) -> &'static str;
}

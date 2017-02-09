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
 * TODO: use macros 2.0 instead for the:
 * * view: to create the dependencies between the view items and the model.
 * * model: to add boolean fields in an inner struct specifying which parts of the view to update
 * *        after the update.
 * * update: to set the boolean fields to true depending on which parts of the model was updated.
 * * create default values for gtk widgets (like Label::new(None)).
 * * create attributes for constructor gtk widgets (like orientation for Box::new(orientation)).
 * TODO: optionnaly multi-threaded.
 */

extern crate futures;
extern crate gtk;
extern crate relm_core;

mod macros;
mod widget;

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::rc::Rc;

use futures::Stream;
use relm_core::{Core, EventStream};

pub use relm_core::QuitFuture;
pub use self::Error::*;
pub use self::widget::*;

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

pub struct Relm<M, W> {
    core: Core<M, W>,
}

impl<M: Clone + 'static, W: 'static> Relm<M, W> {
    pub fn run<D: Widget<M, W> + 'static>(mut widget: D) -> Result<(), Error> {
        gtk::init()?;
        let widgets = Rc::new(widget.view());
        let mut relm = Relm {
            core: Core::new(widgets.clone())?,
        };
        widget.connect_events(&relm, widgets);

        let handle = relm.core.handle();
        let event_future = {
            let stream = relm.stream().clone();
            let quit_future = relm.core.quit_future().clone();
            stream.for_each(move |(event, widgets)| {
                widget.update(event, widgets, &quit_future);
                Ok(())
            })
        };
        handle.spawn(event_future);

        relm.core.run();
        Ok(())
    }

    pub fn stream(&self) -> &EventStream<M, Rc<W>> {
        self.core.stream()
    }
}

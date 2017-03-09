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

#![feature(conservative_impl_trait, fn_traits, unboxed_closures)]

#[macro_use]
extern crate fns_derive;
extern crate futures;
extern crate gdk_pixbuf;
extern crate gtk;
extern crate hyper;
extern crate hyper_tls;
extern crate json;
#[macro_use]
extern crate relm;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use futures::future::ok;
use gdk_pixbuf::PixbufLoader;
use gtk::{Button, ButtonExt, ContainerExt, Image, Label, WidgetExt, Window, WindowType};
use hyper::Client;
use hyper_tls::HttpsConnector;
use gtk::Orientation::Vertical;
use relm::{Handle, QuitFuture, Relm, UnitFuture, Widget};

use self::Msg::*;

#[derive(Clone)]
struct Model {
    gif_url: String,
    topic: String,
}

#[derive(Clone, Fns)]
enum Msg {
    DownloadCompleted,
    FetchUrl,
    ImageChunk(Vec<u8>),
    NewGif(Vec<u8>),
    Quit,
}

struct Widgets {
    image: Image,
    label: Label,
    window: Window,
}

struct Win {
    loader: PixbufLoader,
    model: Model,
    relm: Relm<Msg>,
    widgets: Widgets,
}

impl Win {
    fn view(relm: &Relm<Msg>) -> Widgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let label = Label::new(None);
        vbox.add(&label);

        let image = Image::new();
        vbox.add(&image);

        let button = Button::new_with_label("Load image");
        vbox.add(&button);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, button, connect_clicked(_), FetchUrl);
        connect_no_inhibit!(relm, window, connect_delete_event(_, _), Quit);

        Widgets {
            image: image,
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    type Container = Window;

    fn container(&self) -> &Self::Container {
        &self.widgets.window
    }

    fn new(relm: Relm<Msg>) -> Self {
        let model = Model {
            gif_url: "waiting.gif".to_string(),
            topic: "cats".to_string(),
        };
        let widgets = Self::view(&relm);
        widgets.label.set_text(&model.topic);
        Win {
            loader: PixbufLoader::new(),
            model: model,
            relm: relm,
            widgets: widgets,
        }
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            DownloadCompleted => {
                self.loader.close().unwrap();
                self.widgets.image.set_from_pixbuf(self.loader.get_pixbuf().as_ref());
                self.loader = PixbufLoader::new();
            },
            FetchUrl => {
                let url = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}", self.model.topic);
                let http_future = http_get(&url, self.relm.handle());
                return self.relm.connect(http_future, NewGif);
            },
            NewGif(result) => {
                let string = String::from_utf8(result).unwrap();
                let json = json::parse(&string).unwrap();
                let url = &json["data"]["image_url"].as_str().unwrap();
                let http_future = http_get_stream(url, self.relm.handle());
                let future = self.relm.connect(http_future, ImageChunk);
                return self.relm.connect(future, DownloadCompleted);
            },
            ImageChunk(chunk) => {
                self.loader.loader_write(&chunk).unwrap();
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        // This is necessary to avoid a GDK warning.
        self.loader.close().ok(); // Ignore the error since no data was loaded.
    }
}

fn http_get<'a>(url: &str, handle: &Handle) -> impl Future<Item=Vec<u8>, Error=()> + 'a {
    let stream = http_get_stream(url, handle);
    // TODO: use the new Stream::concat().
    stream.fold(vec![], |mut acc, chunk| {
        acc.extend_from_slice(&chunk);
        Ok(acc)
    })
}

fn http_get_stream<'a>(url: &str, handle: &Handle) -> impl Stream<Item=Vec<u8>, Error=()> + 'a {
    let url = hyper::Url::parse(url).unwrap();
    let connector = HttpsConnector::new(2, handle);
    let client = Client::configure()
        .connector(connector)
        .build(handle);
    client.get(url).and_then(|res| {
        Ok(res.body()
           .map(|chunk| chunk.to_vec())
           .map_err(|_| ())
       )
    })
        .map_err(|_| ())
        .flatten_stream()
}

fn main() {
    Relm::run::<Win>().unwrap();
}

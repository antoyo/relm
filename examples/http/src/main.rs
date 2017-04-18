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

extern crate futures;
extern crate gdk_pixbuf;
extern crate gdk_sys;
extern crate gtk;
extern crate hyper;
extern crate hyper_tls;
extern crate json;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate simplelog;

use std::str::FromStr;

use futures::{Future, Stream};
use gdk_pixbuf::PixbufLoader;
use gdk_sys::GdkRGBA;
use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Image,
    Inhibit,
    Label,
    WidgetExt,
    Window,
    WindowType,
    STATE_FLAG_NORMAL,
};
use gtk::prelude::WidgetExtManual;
use hyper::{Client, Error};
use hyper_tls::HttpsConnector;
use gtk::Orientation::Vertical;
use relm::{Handle, Relm, RemoteRelm, Widget};
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter::Warn;

use self::Msg::*;

const RED: &GdkRGBA = &GdkRGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };

#[derive(Clone)]
struct Model {
    gif_url: String,
    topic: String,
}

#[derive(SimpleMsg)]
enum Msg {
    DownloadCompleted,
    FetchUrl,
    HttpError(String),
    ImageChunk(Vec<u8>),
    NewGif(Vec<u8>),
    Quit,
}

#[derive(Clone)]
struct Win {
    button: Button,
    image: Image,
    label: Label,
    loader: PixbufLoader,
    window: Window,
}

impl Widget for Win {
    type Model = Model;
    type Msg = Msg;
    type Root = Window;

    fn model() -> Model {
        Model {
            gif_url: "waiting.gif".to_string(),
            topic: "cats".to_string(),
        }
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn update(&mut self, event: Msg, _model: &mut Model) {
        match event {
            DownloadCompleted => {
                self.button.set_sensitive(true);
                self.button.grab_focus();
                self.loader.close().unwrap();
                self.image.set_from_pixbuf(self.loader.get_pixbuf().as_ref());
                self.loader = PixbufLoader::new();
            },
            FetchUrl => {
                self.label.set_text("");
                // Disable the button because loading 2 images at the same time crashes the pixbuf
                // loader.
                self.button.set_sensitive(false);
            },
            HttpError(error) => {
                self.button.set_sensitive(true);
                self.label.set_text(&format!("HTTP error: {}", error));
                self.label.override_color(STATE_FLAG_NORMAL, RED);
            },
            ImageChunk(chunk) => {
                self.loader.loader_write(&chunk).unwrap();
            },
            NewGif(_) => (),
            Quit => gtk::main_quit(),
        }
    }

    fn update_command(relm: &Relm<Msg>, event: Msg, model: &mut Model) {
        match event {
            FetchUrl => {
                let url = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}", model.topic);
                let http_future = http_get(&url, relm.handle());
                relm.connect_exec(http_future, NewGif, hyper_error_to_msg);
            },
            NewGif(result) => {
                let string = String::from_utf8(result).unwrap();
                let json = json::parse(&string).unwrap();
                let url = &json["data"]["image_url"].as_str().unwrap();
                let http_future = http_get_stream(url, relm.handle());
                let future = relm.connect(http_future, ImageChunk, hyper_error_to_msg);
                relm.connect_exec_ignore_err(future, DownloadCompleted);
            },
            _ => (),
        }
    }

    fn view(relm: RemoteRelm<Msg>, model: &Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let label = Label::new(None);
        label.set_text(&model.topic);
        vbox.add(&label);

        let image = Image::new();
        vbox.add(&image);

        let button = Button::new_with_label("Load image");
        vbox.add(&button);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, button, connect_clicked(_), FetchUrl);
        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));

        Win {
            button: button,
            image: image,
            label: label,
            loader: PixbufLoader::new(),
            window: window,
        }
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        // This is necessary to avoid a GDK warning.
        self.loader.close().ok(); // Ignore the error since no data was loaded.
    }
}

fn http_get<'a>(url: &str, handle: &Handle) -> impl Future<Item=Vec<u8>, Error=Error> + 'a {
    let stream = http_get_stream(url, handle);
    stream.concat()
}

fn http_get_stream<'a>(url: &str, handle: &Handle) -> impl Stream<Item=Vec<u8>, Error=Error> + 'a {
    let url = hyper::Uri::from_str(url).unwrap();
    let connector = HttpsConnector::new(2, handle);
    let client = Client::configure()
        .connector(connector)
        .build(handle);
    client.get(url).and_then(|res| {
        Ok(res.body()
           .map(|chunk| chunk.to_vec())
       )
    })
        .flatten_stream()
}

#[allow(needless_pass_by_value)]
fn hyper_error_to_msg(error: Error) -> Msg {
    HttpError(error.to_string())
}

fn main() {
    TermLogger::init(Warn, Config::default()).unwrap();
    relm::run::<Win>().unwrap();
}

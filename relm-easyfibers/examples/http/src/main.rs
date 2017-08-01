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

#![feature(conservative_impl_trait, fn_traits, proc_macro, unboxed_closures)]

extern crate futures;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gtk;
extern crate hyper;
extern crate hyper_tls;
extern crate json;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate simplelog;

use std::str::FromStr;

use futures::{Future, Stream};
use gdk::RGBA;
use gdk_pixbuf::PixbufLoader;
use gtk::{
    ButtonExt,
    Inhibit,
    OrientableExt,
    WidgetExt,
    STATE_FLAG_NORMAL,
};
use hyper::{Client, Error};
use hyper_tls::HttpsConnector;
use gtk::Orientation::Vertical;
use relm::{Relm, Widget};
use relm_attributes::widget;
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter::Warn;

use self::Msg::*;

const RED: &RGBA = &RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };

pub struct Model {
    button_enabled: bool,
    gif_url: String,
    loader: PixbufLoader,
    topic: String,
    text: String,
}

#[derive(SimpleMsg)]
pub enum Msg {
    DownloadCompleted,
    FetchUrl,
    HttpError(String),
    ImageChunk(Vec<u8>),
    NewGif(Vec<u8>),
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        let topic = "cats";
        Model {
            button_enabled: true,
            gif_url: "waiting.gif".to_string(),
            loader: PixbufLoader::new(),
            topic: topic.to_string(),
            text: topic.to_string(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            DownloadCompleted => {
                self.model.button_enabled = true;
                self.button.grab_focus();
                self.model.loader.close().unwrap();
                self.image.set_from_pixbuf(self.model.loader.get_pixbuf().as_ref());
                self.model.loader = PixbufLoader::new();
            },
            FetchUrl => {
                self.model.text = String::new();
                // Disable the button because loading 2 images at the same time crashes the pixbuf
                // loader.
                self.model.button_enabled = false;

                let url = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}",
                    self.model.topic);
                let http_future = http_get(&url, relm.handle());
                relm.connect_exec(http_future, NewGif, hyper_error_to_msg);
            },
            HttpError(error) => {
                self.model.button_enabled = true;
                self.model.text = format!("HTTP error: {}", error);
                self.label.override_color(STATE_FLAG_NORMAL, RED);
            },
            ImageChunk(chunk) => {
                self.model.loader.loader_write(&chunk).unwrap();
            },
            NewGif(result) => {
                let string = String::from_utf8(result).unwrap();
                let json = json::parse(&string).unwrap();
                let url = &json["data"]["image_url"].as_str().unwrap();
                let http_future = http_get_stream(url, relm.handle());
                let future = relm.connect(http_future, ImageChunk, hyper_error_to_msg);
                relm.connect_exec_ignore_err(future, DownloadCompleted);
            },
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="label"]
                gtk::Label {
                    text: &self.model.text,
                },
                #[name="image"]
                gtk::Image {
                },
                #[name="button"]
                gtk::Button {
                    label: "Load image",
                    sensitive: self.model.button_enabled,
                    clicked => FetchUrl,
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        // This is necessary to avoid a GDK warning.
        self.model.loader.close().ok(); // Ignore the error since no data was loaded.
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
    Win::run(()).unwrap();
}

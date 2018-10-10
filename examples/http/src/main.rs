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

extern crate glib_sys;

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate json;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate simplelog;
extern crate uhttp_uri;

use std::cell::RefCell;
use std::mem;

use gdk::RGBA;
use gdk_pixbuf::{PixbufLoader, PixbufLoaderExt};
use gio::{
    InputStreamExtManual,
    IOStream,
    IOStreamExt,
    OutputStreamExtManual,
    SocketClient,
    SocketClientExt,
    SocketConnection,
};
use glib::Cast;
use glib::source::PRIORITY_DEFAULT;
use gtk::{
    ButtonExt,
    ImageExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    StateFlags,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{
    Relm,
    Update,
    UpdateNew,
    Widget,
    execute,
};
use relm_attributes::widget;
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter::Warn;
use uhttp_uri::HttpUri;

use self::Msg::*;
use self::HttpMsg::*;

const RED: &RGBA = &RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 };
const READ_SIZE: usize = 1024;

pub struct Model {
    button_enabled: bool,
    loader: PixbufLoader,
    relm: Relm<Win>,
    topic: String,
    text: String,
}

#[derive(Msg)]
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
    fn model(relm: &Relm<Self>, (): ()) -> Model {
        let topic = "cats";
        Model {
            button_enabled: true,
            loader: PixbufLoader::new(),
            relm: relm.clone(),
            topic: topic.to_string(),
            text: topic.to_string(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            DownloadCompleted => {
                self.model.button_enabled = true;
                self.button.grab_focus();
                self.model.loader.close().ok();
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
                let http = execute::<Http>(url);
                connect_stream!(http@ReadDone(ref buffer), self.model.relm.stream(), NewGif(buffer.take()));
            },
            HttpError(error) => {
                self.model.button_enabled = true;
                self.model.text = format!("HTTP error: {}", error);
                self.label.override_color(StateFlags::NORMAL, RED);
            },
            ImageChunk(chunk) => {
                //self.model.loader.write(&chunk).expect("write failure");
                if let Err(error) = self.model.loader.write(&chunk) {
                    println!("{}", error);
                }
            },
            NewGif(buffer) => {
                if let Ok(body) = String::from_utf8(buffer) {
                    let mut json = json::parse(&body).expect("invalid json");
                    let url = json["data"]["image_url"].take_string().expect("take_string failed");
                    let http = execute::<Http>(url);
                    connect_stream!(http@DataRead(ref buffer), self.model.relm.stream(), ImageChunk(buffer.take()));
                    connect_stream!(http@ReadDone(_), self.model.relm.stream(), DownloadCompleted);
                }
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

struct HttpModel {
    buffer: Vec<u8>,
    found_crlf: bool,
    relm: Relm<Http>,
    stream: Option<IOStream>,
    url: String,
}

struct Bytes {
    bytes: RefCell<Option<Vec<u8>>>,
}

impl Bytes {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: RefCell::new(Some(bytes)),
        }
    }

    fn take(&self) -> Vec<u8> {
        self.bytes.borrow_mut().take().unwrap_or_default()
    }
}

#[derive(Msg)]
enum HttpMsg {
    Connection(SocketConnection),
    DataRead(Bytes),
    Read((Vec<u8>, usize)),
    ReadDone(Bytes),
    Wrote,
}

unsafe impl Send for HttpMsg {}

struct Http {
    model: HttpModel,
}

impl Update for Http {
    type Model = HttpModel;
    type ModelParam = String;
    type Msg = HttpMsg;

    fn model(relm: &Relm<Self>, url: String) -> HttpModel {
        HttpModel {
            buffer: vec![],
            found_crlf: false,
            stream: None,
            relm: relm.clone(),
            url,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let client = SocketClient::new();
        // TODO: call client.set_tls().
        if let Some(host) = HttpUri::new(&self.model.url).ok().map(|uri| uri.authority) {
            connect_async!(client, connect_to_host_async(host, 80), relm, Connection);
        }
    }

    fn update(&mut self, message: HttpMsg) {
        match message {
            Connection(connection) => {
                let stream: IOStream = connection.upcast();
                let writer = stream.get_output_stream().expect("output");
                self.model.stream = Some(stream);
                if let Ok(uri) = HttpUri::new(&self.model.url) {
                    let path = uri.resource.path;
                    let query = uri.resource.query.unwrap_or_default();
                    let buffer = format!("GET {}?{}\r\nHost: {}\r\n\r\n", path, query, uri.authority);
                    connect_async!(writer, write_async(buffer.into_bytes(), PRIORITY_DEFAULT), self.model.relm,
                        |_| Wrote);
                }
            },
            // To be listened by the user.
            DataRead(_) => (),
            Read((mut buffer, size)) => {
                if size == 0 {
                    let buffer = mem::replace(&mut self.model.buffer, vec![]);
                    self.model.relm.stream().emit(ReadDone(Bytes::new(buffer)));
                }
                else {
                    if let Some(ref stream) = self.model.stream {
                        let reader = stream.get_input_stream().expect("input");
                        connect_async!(reader, read_async(vec![0; READ_SIZE], PRIORITY_DEFAULT), self.model.relm, Read);
                    }
                }
                buffer.truncate(size);
                let buffer =
                    if self.model.found_crlf {
                        buffer
                    }
                    else if let Some(index) = find_crlf(&buffer) {
                        self.model.found_crlf = true;
                        buffer[index + 4..].to_vec()
                    }
                    else {
                        vec![]
                    };
                self.model.buffer.extend(&buffer);
                self.model.relm.stream().emit(DataRead(Bytes::new(buffer)));
            },
            // To be listened by the user.
            ReadDone(_) => (),
            Wrote => {
                if let Some(ref stream) = self.model.stream {
                    let reader = stream.get_input_stream().expect("input");
                    connect_async!(reader, read_async(vec![0; READ_SIZE], PRIORITY_DEFAULT), self.model.relm, Read);
                }
            },
        }
    }
}

impl UpdateNew for Http {
    fn new(_relm: &Relm<Self>, model: HttpModel) -> Self {
        Http {
            model,
        }
    }
}

fn find_crlf(buffer: &[u8]) -> Option<usize> {
    for i in 0..buffer.len() {
        if buffer[i..].len() < 4 {
            return None;
        }
        if &buffer[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

fn main() {
    TermLogger::init(Warn, Config::default()).expect("TermLogger::init failed");
    Win::run(()).expect("Win::run failed");
}

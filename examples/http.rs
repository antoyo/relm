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

#![feature(conservative_impl_trait)]

extern crate futures;
extern crate gdk_pixbuf;
extern crate gtk;
extern crate json;
#[macro_use]
extern crate relm;
extern crate tokio_core;
extern crate url;

use std::io::Error;
use std::net::ToSocketAddrs;

use futures::Future;
use futures::future::ok;
use gdk_pixbuf::PixbufLoader;
use gtk::{Button, ButtonExt, ContainerExt, Image, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm::{EventStream, Handle, QuitFuture, Relm, UnitFuture, Widget, connect};
use tokio_core::net::TcpStream;
use url::Url;

use self::Msg::*;

#[derive(Clone, Debug)]
struct Model {
    gif_url: String,
    topic: String,
}

#[derive(Clone)]
enum Msg {
    FetchUrl,
    NewGif(Vec<u8>),
    NewImage(Vec<u8>),
    Quit,
}

struct Widgets {
    button: Button,
    image: Image,
    label: Label,
    window: Window,
}

struct Win {
    handle: Handle,
    model: Model,
    stream: EventStream<Msg>,
    widgets: Widgets,
}

impl Win {
    fn view() -> Widgets {
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

        Widgets {
            button: button,
            image: image,
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    fn connect_events(&self, relm: &Relm<Msg>) {
        connect!(relm, self.widgets.button, connect_clicked(_), FetchUrl);
        connect_no_inhibit!(relm, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn new(handle: Handle, stream: EventStream<Msg>) -> Self {
        let model = Model {
            gif_url: "waiting.gif".to_string(),
            topic: "cats".to_string(),
        };
        let widgets = Self::view();
        widgets.label.set_text(&model.topic);
        Win {
            handle: handle,
            model: model,
            stream: stream,
            widgets: widgets,
        }
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            FetchUrl => {
                let url = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}", self.model.topic);
                //let url = format!("https://api.giphy.com/v1/gifs"); // TODO: test with this URL because it freezes the UI.
                let http_future = http_get(&url, &self.handle);
                return connect(http_future, NewGif, &self.stream);
            },
            NewGif(result) => {
                let string = String::from_utf8(result).unwrap();
                let json = json::parse(&string).unwrap();
                let url = &json["data"]["image_url"].as_str().unwrap();
                let http_future = http_get(url, &self.handle);
                return connect(http_future, NewImage, &self.stream);
            },
            NewImage(result) => {
                let loader = PixbufLoader::new();
                loader.loader_write(&result).unwrap();
                loader.close().unwrap();
                self.widgets.image.set_from_pixbuf(loader.get_pixbuf().as_ref());
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn http_get<'a>(url: &str, handle: &Handle) -> impl Future<Item=Vec<u8>, Error=Error> + 'a {
    let url = Url::parse(url).unwrap();
    let path = format!("{}?{}", url.path(), url.query().unwrap_or(""));
    let url = url.host_str();
    let url = url.unwrap();
    let host = format!("{}:80", url);
    let addr = host.to_socket_addrs().unwrap().next().unwrap();
    let socket = TcpStream::connect(&addr, handle);
    let http = format!("\
        GET {} HTTP/1.0\r\n\
        Host: {}\r\n\
        \r\n\
    ", path, url);
    let request = socket.and_then(move |socket| {
        tokio_core::io::write_all(socket, http.into_bytes())
    });

    let response = request.and_then(|(socket, _request)| {
        tokio_core::io::read_to_end(socket, Vec::new())
    });
    response.and_then(|(_socket, response)| {
        let mut index = 0;
        for i in 5..response.len() {
            if &response[i - 4..i] == b"\r\n\r\n" {
                index = i;
                break;
            }
        }
        let body = &response[index..];
        ok(body.to_vec())
    })
}

fn main() {
    Relm::run::<Win>().unwrap();
}

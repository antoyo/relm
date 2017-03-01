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
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate tokio_core;
extern crate url;

use std::net::ToSocketAddrs;

use futures::Future;
use futures::future::ok;
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
    FetchImage,
    FetchUrl,
    NewGif(String),
    Quit,
}

struct Widgets {
    button: Button,
    image: Image,
    label: Label,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Win {
    fn view() -> Widgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let label = Label::new(None);
        vbox.add(&label);

        let image = Image::new_from_resource("https://k1.okccdn.com/php/load_okc_image.php/images/120x120/120x120/0x65/531x597/2/6119803335702555186.jpeg?v=1");
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

    fn new() -> Self {
        Win {
            model: Model {
                gif_url: "waiting.gif".to_string(),
                topic: "cats".to_string(),
            },
            widgets: Self::view(),
        }
    }

    fn update(&mut self, event: Msg, handle: Handle, stream: EventStream<Msg>) -> UnitFuture {
        match event {
            FetchImage => {
            },
            FetchUrl => {
                let url = format!("https://api.giphy.com/v1/gifs/random?api_key=dc6zaTOxFJmzC&tag={}", self.model.topic);
                let http_future = http_get(&url, handle);
                return connect(http_future, NewGif, stream).boxed();
            },
            NewGif(result) => {
                println!("{}", result);
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn http_get<'a>(url: &str, handle: Handle) -> impl Future<Item=String, Error=()> + 'a {
    let url = Url::parse(url).unwrap();
    let path = format!("{}?{}", url.path(), url.query().unwrap_or(""));
    let url = url.host_str();
    let url = url.unwrap();
    let host = format!("{}:80", url);
    let addr = host.to_socket_addrs().unwrap().next().unwrap();
    let socket = TcpStream::connect(&addr, &handle);
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
        let string = String::from_utf8(response).unwrap();
        let strings: Vec<_> = string.split("\n\n").collect();
        let body = strings[1].to_string();
        ok(body)
    })
        // TODO: try to box (so that it becomes Send) the error to keep it instead of ignoring it.
        .map_err(|_| ())
}

fn main() {
    Relm::run::<Win>().unwrap();
}

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

extern crate base64;
extern crate byteorder;
extern crate futures;
extern crate gtk;
extern crate json;
extern crate rand;
#[macro_use]
extern crate relm;
extern crate slog_stdlog;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate twist;
extern crate url;

use std::net::ToSocketAddrs;

use base64::encode;
use byteorder::{BigEndian, WriteBytesExt};
use futures::Future;
use futures::future::ok;
use gtk::{Button, ButtonExt, ContainerExt, Entry, EntryExt, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use rand::Rng;
use relm::{EventStream, Handle, QuitFuture, Relm, UnitFuture, Widget, connect};
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_proto::pipeline::ClientService;
use tokio_service::Service;
use twist::client::{BaseFrame, HandshakeRequestFrame, OpCode, WebSocketFrame, WebSocketProtocol};

use self::Msg::*;

type WSService = ClientService<TcpStream, WebSocketProtocol>;

#[derive(Clone, Debug)]
struct Model {
    text: String,
}

#[derive(Clone)]
enum Msg {
    Connected(WSService),
    Message(String),
    Send,
    Quit,
}

struct Widgets {
    button: Button,
    entry: Entry,
    label: Label,
    window: Window,
}

struct Win {
    model: Model,
    service: Option<WSService>,
    stream: EventStream<Msg>,
    widgets: Widgets,
}

impl Win {
    fn view() -> Widgets {
        let vbox = gtk::Box::new(Vertical, 0);

        let label = Label::new(None);
        vbox.add(&label);

        let entry = Entry::new();
        vbox.add(&entry);

        let button = Button::new_with_label("Send");
        vbox.add(&button);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        Widgets {
            button: button,
            entry: entry,
            label: label,
            window: window,
        }
    }
}

impl Widget<Msg> for Win {
    fn connect_events(&self, relm: &Relm<Msg>) {
        connect!(relm, self.widgets.entry, connect_activate(_), Send);
        connect!(relm, self.widgets.button, connect_clicked(_), Send);
        connect_no_inhibit!(relm, self.widgets.window, connect_delete_event(_, _), Quit);
    }

    fn new(handle: Handle, stream: EventStream<Msg>) -> Self {
        let model = Model {
            text: String::new(),
        };

        let handshake_future = ws_handshake(&handle);
        let future = connect(handshake_future, Connected, &stream);
        handle.spawn(future);

        let widgets = Self::view();
        Win {
            model: model,
            service: None,
            stream: stream,
            widgets: widgets,
        }
    }

    fn update(&mut self, event: Msg) -> UnitFuture {
        match event {
            Connected(service) => {
                self.service = Some(service);
            },
            Message(message) => {
                self.model.text.push_str(&format!("{}\n", message));
                self.widgets.label.set_text(&self.model.text);
            },
            Send => {
                if let Some(ref service) = self.service {
                    let message = self.widgets.entry.get_text().unwrap_or_else(String::new);
                    self.widgets.entry.set_text("");
                    self.widgets.entry.grab_focus();
                    let send_future = ws_send(service, &message);
                    return connect(send_future, Message, &self.stream);
                }
            },
            Quit => return QuitFuture.boxed(),
        }

        ok(()).boxed()
    }
}

fn gen_nonce() -> String {
    let mut rng = rand::thread_rng();
    let mut nonce_vec = Vec::with_capacity(2);
    let nonce = rng.gen::<u16>();

    if nonce_vec.write_u16::<BigEndian>(nonce).is_ok() {
        encode(&nonce_vec)
    } else {
        nonce_vec.clear();
        nonce_vec.push(rng.gen::<u8>());
        nonce_vec.push(rng.gen::<u8>());
        encode(&nonce_vec)
    }
}

fn ws_handshake(handle: &Handle) -> impl Future<Item=WSService> {
    let mut protocol = WebSocketProtocol::default();
    protocol.client(true);
    let client = TcpClient::new(protocol);
    let url = "echo.websocket.org:80";
    client.connect(&url.to_socket_addrs().unwrap().next().unwrap(), handle)
        .and_then(|service| {
            let nonce = gen_nonce();
            let mut handshake_frame = WebSocketFrame::default();
            let mut handshake = HandshakeRequestFrame::default();
            handshake.set_user_agent("twisty 0.1.0".to_string());
            handshake.set_origin("http://www.websocket.org".to_string());
            handshake.set_host("echo.websocket.org".to_string());
            handshake.set_sec_websocket_key(nonce.to_string());
            handshake_frame.set_clientside_handshake_request(handshake);
            service.call(handshake_frame)
                .map(|_socket| service)
        })
}

fn ws_send(service: &WSService, message: &str) -> impl Future<Item=String> {
    let mut frame = WebSocketFrame::default();
    let mut base = BaseFrame::default();
    base.set_fin(true);
    base.set_masked(true);
    base.set_mask(0);
    base.set_opcode(OpCode::Text);
    base.set_payload_length(message.len() as u64);
    base.set_application_data(Some(message.as_bytes().to_vec()));
    frame.set_base(base);
    service.call(frame)
        .and_then(|socket| {
            let bytes = socket.base().unwrap().application_data().unwrap();
            let string = String::from_utf8_lossy(bytes).to_string();
            Ok(string)
        })
}

fn main() {
    Relm::run::<Win>().unwrap();
}

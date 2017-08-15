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

#![feature(conservative_impl_trait, proc_macro)]

extern crate base64;
extern crate blake2;
extern crate byteorder;
extern crate futures;
extern crate gtk;
extern crate rand;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate twist;

use std::net::ToSocketAddrs;

use base64::encode;
use blake2::{Blake2b, Digest};
use byteorder::{BigEndian, WriteBytesExt};
use futures::Future;
use gtk::{
    ButtonExt,
    EditableSignals,
    EntryExt,
    Inhibit,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use rand::Rng;
use relm::{Handle, Relm, Widget};
use relm_attributes::widget;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_proto::pipeline::ClientService;
use tokio_service::Service;
use twist::client::{BaseFrame, HandshakeRequestFrame, OpCode, WebSocketFrame, WebSocketProtocol};

use self::Msg::*;

type WSService = ClientService<TcpStream, WebSocketProtocol>;

pub struct Model {
    // The message to be sent.
    message: String,
    service: Option<WSService>,
    // This contains all the messages received from the websockets server.
    text: String,
}

#[derive(Msg)]
pub enum Msg {
    // The user changed the message to be sent.
    Change(String),
    // Connection to the server successful.
    Connected(WSService),
    // A message received from the server.
    Message(String),
    // Send a message to the server.
    Send,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model {
            message: String::new(),
            service: None,
            text: String::new(),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        // Connect to the websocket server.
        let handshake_future = ws_handshake(relm.handle());
        let future = relm.connect_ignore_err(handshake_future, Connected);
        relm.exec(future);
    }

    fn update(&mut self, event: Msg, model: &mut Model) {
        match event {
            Change(message) => model.message = message,
            Connected(service) => model.service = Some(service),
            Message(message) => model.text += &format!("{}\n", message),
            Send => {
                model.message = String::new();
                self.entry.grab_focus();
            },
            Quit => gtk::main_quit(),
        }
    }

    fn update_command(relm: &Relm<Self>, event: Msg, model: &mut Model) {
        if let Send = event {
            if let Some(ref service) = model.service {
                // Send the message to the server.
                let send_future = ws_send(service, &model.message);
                relm.connect_exec_ignore_err(send_future, Message);
            }
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                gtk::Label {
                    text: &model.text,
                },
                // Give a name to this widget, so that we can use it in the update function.
                #[name="entry"]
                gtk::Entry {
                    activate => Send,
                    changed(entry) => Change(entry.get_text().unwrap_or_else(String::new)),
                    text: &model.message,
                },
                gtk::Button {
                    clicked => Send,
                    label: "Send",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn gen_nonce() -> String {
    let mut rng = rand::thread_rng();
    let mut nonce_vec = Vec::with_capacity(2);
    let nonce = rng.gen::<u16>();
    let mut hasher = Blake2b::default();

    if nonce_vec.write_u16::<BigEndian>(nonce).is_ok() {
        hasher.input(&nonce_vec);
        encode(&hasher.result())
    } else {
        nonce_vec.clear();
        nonce_vec.push(rng.gen::<u8>());
        nonce_vec.push(rng.gen::<u8>());
        hasher.input(&nonce_vec);
        encode(&hasher.result())
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
            handshake.set_path("/".to_string());
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
    Win::run(()).unwrap();
}

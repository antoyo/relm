use glib::Propagation;
use gtk::prelude::*;
use gtk::Adjustment;
use relm::{Relm, StreamHandle, Widget};
use relm_derive::{widget, Msg};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// The pulse time milli seconds.
const PULSE_TIME: u64 = 50;

/// The messages sent to the timer thread.
pub enum ThreadMsg {
    Start,
    Stop,
    Quit,
}

#[derive(Msg, Debug)]
pub enum Msg {
    Start,
    Stop,
    Pulse,
    Reset,
    SetMax(f64),
    Quit,
}

pub struct Model {
    max_time: f64,
    current_time: f64,

    label_text: String,

    // This `StreamHandle` can be used to send messages to the own widget.
    msg_stream: StreamHandle<Msg>,

    // The sender for communication to the calculation thread.
    thread_send: mpsc::Sender<ThreadMsg>,
}

#[widget]
impl Widget for Win {
    /// The model function can take two optional arguments.
    /// The first argument is the relm used for sending messages to the widget.
    /// The second argument can be any object you need for the creation of the model.
    /// This argument is not used in this example.
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        // The Channel for sending from the main thread to the timer thread.
        // Used for communicating start/pause/quit.
        let (thread_send, thread_rec) = mpsc::channel();

        // This is the stream for sending messages to the own widget.
        let stream = relm.stream().clone();

        // The Channel for sending from the timer thread to the main thread.
        // Used for communicating a pulse.
        let (_channel, sender) = relm::Channel::new(move |msg| {
            stream.emit(msg);
        });

        // The timer thread.
        thread::spawn(move || {
            let mut quit = false;

            // Wait for message when running.
            while !quit {
                match thread_rec.recv() {
                    // Start the timer.
                    Ok(ThreadMsg::Start) => {
                        let mut running = true;
                        while running {
                            // Try receiving. When nothing is received, sleep and send the pulse.
                            match thread_rec.try_recv() {
                                Ok(ThreadMsg::Start) => {}
                                Ok(ThreadMsg::Stop) => running = false,
                                Ok(ThreadMsg::Quit) => {
                                    running = false;
                                    quit = true;
                                }
                                Err(_) => {
                                    thread::sleep(Duration::from_millis(PULSE_TIME));
                                    sender.send(Msg::Pulse).expect("Could not send pulse");
                                }
                            }
                        }
                    }
                    Ok(ThreadMsg::Stop) => {}
                    Ok(ThreadMsg::Quit) => quit = true,
                    Err(_) => {}
                }
            }
        });

        relm.stream().clone().emit(Msg::Start);

        Model {
            max_time: 10.0,
            current_time: 0.0,
            label_text: "".to_string(),

            msg_stream: relm.stream().clone(),
            thread_send,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            // Start the timer thread.
            Msg::Start => {
                self.model
                    .thread_send
                    .send(ThreadMsg::Start)
                    .expect("Could not send message to timer thread");
            }
            // Stop the timer thread.
            Msg::Stop => {
                self.model
                    .thread_send
                    .send(ThreadMsg::Stop)
                    .expect("Could not send message to timer thread");
            }
            // Increment the current time.
            Msg::Pulse => {
                self.model.current_time += (PULSE_TIME as f64) / 1000.0;
                if self.model.current_time > self.model.max_time {
                    self.model.current_time = self.model.max_time;
                    self.model.msg_stream.emit(Msg::Stop);
                }

                self.model.label_text = format!("{:.2}s", self.model.current_time);
            }
            // Reset the timer to 0.
            Msg::Reset => {
                self.model.current_time = 0.0;
                self.model.msg_stream.emit(Msg::Start)
            }
            // Set the new maximum.
            Msg::SetMax(max) => {
                self.model.max_time = max;
                if self.model.current_time > self.model.max_time {
                    self.model.msg_stream.emit(Msg::Stop);
                } else {
                    self.model.msg_stream.emit(Msg::Start);
                }
            }
            // Quit the application
            Msg::Quit => {
                self.model
                    .thread_send
                    .send(ThreadMsg::Quit)
                    .expect("Could not send message to timer thread");
                gtk::main_quit();
            }
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                gtk::ProgressBar {
                    fraction: self.model.current_time / self.model.max_time,
                },
                gtk::Label {
                    text: &self.model.label_text
                },
                gtk::Scale {
                    adjustment: &Adjustment::new(10.0, 0.0, 100.0, 1.0, 10.0, 0.0),
                    value_changed(scale) => {
                        let value = scale.value();
                        Msg::SetMax(value)
                    }
                },
                gtk::Button {
                    clicked => Msg::Reset,
                    label: "Reset"
                }
            },
            delete_event(_, _) => (Msg::Quit, Propagation::Proceed),
        }
    }
}

fn main() {
    Win::run(()).expect("Could not spawn window");
}

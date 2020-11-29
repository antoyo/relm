/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

use std::f64::consts::PI;

use gdk::{EventMask, RGBA};
use gtk::{
    BoxExt,
    DrawingArea,
    Inhibit,
    OrientableExt,
    WidgetExt,
    prelude::WidgetExtManual,
};
use gtk::Orientation::Vertical;
use rand::Rng;
use relm_derive::Msg;
use relm::{
    DrawHandler,
    Relm,
    Widget,
    interval,
};
use relm_derive::widget;

use self::Msg::*;

const SIZE: f64 = 15.0;

struct Circle {
    x: f64,
    y: f64,
    color: RGBA,
    vx: f64,
    vy: f64,
}

impl Circle {
    fn generate() -> Self {
        let mut gen = rand::thread_rng();
        Circle {
            x: gen.gen_range(20.0, 500.0),
            y: gen.gen_range(20.0, 500.0),
            color: RGBA {
                red: gen.gen_range(0.0, 1.0),
                green: gen.gen_range(0.0, 1.0),
                blue: gen.gen_range(0.0, 1.0),
                alpha: 1.0,
            },
            vx: gen.gen_range(1.0, 5.0),
            vy: gen.gen_range(1.0, 5.0),
        }
    }
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    circles: Vec<Circle>,
    cursor_pos: (f64, f64),
}

#[derive(Msg)]
pub enum Msg {
    Generate,
    Move,
    MoveCursor((f64, f64)),
    Quit,
    UpdateDrawBuffer,
}

#[widget]
impl Widget for Win {
    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.widgets.drawing_area);
        self.widgets.drawing_area.add_events(EventMask::POINTER_MOTION_MASK);
    }

    fn model() -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            circles: vec![Circle::generate()],
            cursor_pos: (-1000.0, -1000.0),
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Generate);
        interval(relm.stream(), 16, || Move);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Generate => self.model.circles.push(Circle::generate()),
            Move => {
                let allocation = self.widgets.drawing_area.get_allocation();
                for circle in &mut self.model.circles {
                    if (circle.x + circle.vx + SIZE / 2.0 < allocation.width as f64) &&
                        (circle.x + circle.vx - SIZE / 2.0 > 0.0)
                    {
                        circle.x += circle.vx;
                    }
                    else {
                        circle.vx *= -1.0;
                    }
                    if (circle.y + circle.vy + SIZE / 2.0 < allocation.height as f64) &&
                        (circle.y + circle.vy - SIZE / 2.0 > 0.0)
                    {
                        circle.y += circle.vy;
                    }
                    else {
                        circle.vy *= -1.0;
                    }
                }
            },
            MoveCursor(pos) => self.model.cursor_pos = pos,
            Quit => gtk::main_quit(),
            UpdateDrawBuffer => {
                let context = self.model.draw_handler.get_context();
                context.set_source_rgb(1.0, 1.0, 1.0);
                context.paint();
                for circle in &self.model.circles {
                    context.set_source_rgb(circle.color.red, circle.color.green, circle.color.blue);
                    context.arc(circle.x, circle.y, SIZE, 0.0, 2.0 * PI);
                    context.fill();
                }
                context.set_source_rgb(0.1, 0.2, 0.3);
                context.rectangle(self.model.cursor_pos.0 - SIZE / 2.0, self.model.cursor_pos.1 - SIZE / 2.0, SIZE,
                    SIZE);
                context.fill();
            },
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="drawing_area"]
                gtk::DrawingArea {
                    child: {
                        expand: true,
                    },
                    draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
                    motion_notify_event(_, event) => (MoveCursor(event.get_position()), Inhibit(false))
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}

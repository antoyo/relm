use crate::gui::popup_menu::{PopupMenu, PopupMenuMsg};
use crate::gui::win::WinMsg;
use crate::gui::window_resize::{WindowResize, WindowResizeMsg};
use crate::model::{Circle, CircleGroup};

use gdk::{EventButton, EventMotion};
use gtk::{
    prelude::WidgetExtManual, ContainerExt, DrawingArea, EventBox, GtkWindowExt, Inhibit, WidgetExt,
};
use relm::{connect, init, Component, DrawHandler, Relm, StreamHandle, Update, Widget};
use relm_derive::Msg;

const STARTING_RADIUS: u64 = 50;

const MOUSE_LEFT_CLICK: u32 = 1;
const MOUSE_RIGHT_CLICK: u32 = 3;

#[derive(Msg)]
pub enum CircleDrawingMsg {
    /// This will be called from the popup menu.
    StartResize,
    /// This will be called when closing the resize window.
    StopResize,
    /// This will be called when moving the slider in the resize window.
    Resize(u64),

    /// The canvas has been clicked.
    Clicked(EventButton),
    /// The mouse has moved on the canvas.
    MouseMove(EventMotion),

    /// This will be called from the main window when undoing/redoing.
    SetCircles(CircleGroup),

    /// This will redraw the drawing area
    UpdateDrawBuffer,
}

pub struct CircleDrawingModel {
    circles: CircleGroup,

    /// Whether a circle is selected by right-clicking.
    is_circle_selected: bool,

    win_stream: StreamHandle<WinMsg>,

    /// The `DrawHandler` manages the drawing operations of a widget.
    draw_handler: DrawHandler<DrawingArea>,
}

/// The widget for drawing the circles with clicking.
pub struct CircleDrawing {
    model: CircleDrawingModel,

    relm: Relm<Self>,

    drawing_area: DrawingArea,
    event_box: EventBox,
    popup_menu: Component<PopupMenu>,
    resize_window: Option<Component<WindowResize>>,
}

/// You can also implement `relm::Widget` manually by implementing `relm::Update` and `relm::Widget`.
/// This will give you more control, but is more work.
impl Update for CircleDrawing {
    type Model = CircleDrawingModel;
    type ModelParam = StreamHandle<WinMsg>;
    type Msg = CircleDrawingMsg;

    fn model(_relm: &Relm<Self>, win_stream: StreamHandle<WinMsg>) -> CircleDrawingModel {
        CircleDrawingModel {
            circles: CircleGroup::new(),
            is_circle_selected: false,

            win_stream,

            // Create a new `DrawHandler`. It receives the widget in the `view` method.`
            draw_handler: DrawHandler::new().expect("Could not create draw handler"),
        }
    }

    fn update<'a>(&mut self, event: CircleDrawingMsg) {
        match event {
            CircleDrawingMsg::StartResize => {
                self.model.is_circle_selected = true;
                let selected = self.model.circles.get_selected().clone();
                let radius = selected.map(|c| c.get_radius()).unwrap_or(100);

                // Spawn the window with the slider.
                self.resize_window = Some(
                    init::<WindowResize>((self.relm.stream().clone(), radius))
                        .expect("could not spawn secondary window"),
                );
            }
            CircleDrawingMsg::StopResize => {
                self.model.is_circle_selected = false;
                self.resize_window = None;

                // This method will ask gtk to redraw the drawing area.
                self.drawing_area.queue_draw();

                self.emit_new_circle_group();
            }
            CircleDrawingMsg::Resize(radius) => {
                self.model.circles.resize_selected(radius);
                self.drawing_area.queue_draw();
            }
            CircleDrawingMsg::Clicked(button_event) => {
                let (pos_x, pos_y) = button_event.get_position();
                let button = button_event.get_button();

                if button == MOUSE_LEFT_CLICK {
                    // Add a new circle on left click.
                    self.model.circles.add(Circle::new(
                        pos_x as u64,
                        pos_y as u64,
                        STARTING_RADIUS,
                    ));
                    self.drawing_area.queue_draw();
                    self.emit_new_circle_group();
                } else if button == MOUSE_RIGHT_CLICK {
                    // Edit the radius of the circle on right click.
                    if let Some(_circle) = self.model.circles.get_selected() {
                        // Spawn the popup menu when the click happened on a circle.
                        self.popup_menu
                            .emit(PopupMenuMsg::ShowAt(pos_x as u64, pos_y as u64));
                    }
                }
            }
            CircleDrawingMsg::MouseMove(motion_event) => {
                // Only update the hovered circle if no circle was selected by right-clicking.
                if !self.model.is_circle_selected {
                    let (pos_x, pos_y) = motion_event.get_position();
                    self.model.circles.select_at(pos_x as u64, pos_y as u64);
                    self.drawing_area.queue_draw();
                }
            }

            CircleDrawingMsg::SetCircles(circles) => {
                // Reset the shown circles.
                self.model.circles = circles;
                if let Some(win) = &self.resize_window {
                    win.emit(WindowResizeMsg::Quit);
                    win.widget().close();
                } else {
                    self.model.is_circle_selected = false;
                    self.drawing_area.queue_draw();
                }
            }
            CircleDrawingMsg::UpdateDrawBuffer => {
                let context = self.model.draw_handler.get_context();
                let drawing_area = &self.drawing_area;
                let circles = &self.model.circles;

                // Draw the background
                context.rectangle(
                    0.0,
                    0.0,
                    drawing_area.get_allocated_width() as f64,
                    drawing_area.get_allocated_height() as f64,
                );

                context.set_source_rgb(1.0, 1.0, 1.0);
                context.fill();

                // Drawing the circles.
                for circle in circles.get_all() {
                    // The circle to be drawn.
                    context.arc(
                        circle.get_x() as f64,
                        circle.get_y() as f64,
                        circle.get_radius() as f64,
                        0.0,
                        2.0 * std::f64::consts::PI,
                    );

                    // Draw fill if selected.
                    if circle.is_selected() {
                        context.set_source_rgb(0.5, 0.5, 0.5);
                        context.fill_preserve();
                    }

                    context.set_source_rgb(0.0, 0.0, 0.0);
                    context.stroke();
                }
            }
        }
    }
}

impl Widget for CircleDrawing {
    type Root = EventBox;

    fn root(&self) -> Self::Root {
        self.event_box.clone()
    }

    fn view(relm: &Relm<Self>, mut model: Self::Model) -> Self {
        let event_box = EventBox::new();
        let drawing_area = DrawingArea::new();
        event_box.add(&drawing_area);

        connect!(
            relm,               // The `Relm` to send messages to
            drawing_area,       // The `gtk::Widget`.
            connect_draw(_, _), // The event to connect to.
            return (
                // What to do when the signal occured.
                Some(CircleDrawingMsg::UpdateDrawBuffer), // Sending a message.
                Inhibit(false) // Do not inhibit, other widgets may also register that mouse press.
            )
        );

        // Connect mouse clicking.
        connect!(
            relm,                                 // The `Relm` to send messages to
            event_box,                            // The `gtk::Widget`.
            connect_button_press_event(_, event), // The event to connect to.
            return (
                // What to do when the signal occured.
                Some(CircleDrawingMsg::Clicked(event.clone())), // Sending a message.
                Inhibit(false) // Do not inhibit, other widgets may also register that mouse press.
            )
        );

        // `event_box` must receive all signals.
        // Deleting this line would only send the `MouseMove` message when dragging.
        event_box.set_events(gdk::EventMask::all());

        // Connect the mouse movement.
        connect!(
            relm,
            event_box,
            connect_motion_notify_event(_, event),
            return (
                Some(CircleDrawingMsg::MouseMove(event.clone())),
                Inhibit(false)
            )
        );

        // Show everything.
        event_box.show_all();

        // Give the draw handler the widget.
        model.draw_handler.init(&drawing_area);

        CircleDrawing {
            model,

            relm: relm.clone(),

            event_box,
            drawing_area: drawing_area.clone(),
            popup_menu: relm::create_component((relm.stream().clone(), drawing_area)),
            resize_window: None,
        }
    }
}

impl CircleDrawing {
    /// Send a message to the window.
    /// This will be called when the `self.model.circles` has significant changed like
    /// a new circle or a finalized resize.
    fn emit_new_circle_group(&self) {
        let mut circle_group = self.model.circles.clone();
        circle_group.delesect_all();
        self.model
            .win_stream
            .emit(WinMsg::AddCircleGroup(circle_group));
    }
}

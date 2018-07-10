//! Utility to help drawing on a widget in a relm application.
//! Create a DrawHandler, initialize it, and get its context when handling a message (that could be
//! sent from the draw signal).

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use cairo::{
    self,
    Context,
    Format,
    ImageSurface,
    Rectangle,
};
use gtk::{
    Inhibit,
    WidgetExt,
};

#[derive(Clone)]
struct Surface {
    surface: Rc<RefCell<ImageSurface>>,
}

impl Surface {
    fn new(surface: ImageSurface) -> Self {
        Self {
            surface: Rc::new(RefCell::new(surface)),
        }
    }

    fn get(&self) -> ImageSurface {
        self.surface.borrow().clone()
    }

    fn set(&self, surface: &ImageSurface) {
        *self.surface.borrow_mut() = surface.clone();
    }
}

pub struct DrawContext<W: WidgetExt> {
    //clip_rectangles: RectangleVec,
    context: Context,
    draw_surface: Surface,
    edit_surface: ImageSurface,
    widget: W,
}

impl<W: Clone + WidgetExt> DrawContext<W> {
    fn new(draw_surface: &Surface, edit_surface: &ImageSurface, widget: &W, clip_rectangles: &[Rectangle]) -> Self {
        let context = Context::new(&edit_surface);
        //context.identity_matrix(); // FIXME: not sure it's needed.
        // FIXME: don't call queue_draw(), provide an API to do manual clipping and draw whenever
        // it's required (i.e. in the motion_notify signal, for instance).
        for rect in clip_rectangles {
            //println!("1. {}, {}", rect.width, rect.height);
            context.rectangle(rect.x, rect.y, rect.width, rect.height);
            /*context.move_to(rect.x, rect.y);
              context.rel_line_to(rect.width, 0.0);
              context.rel_line_to(0.0, rect.height);
              context.rel_line_to(-rect.width, 0.0);
              context.rel_line_to(0.0, -rect.height);
              context.close_path();*/
        }
        if !clip_rectangles.is_empty() {
            context.clip();
        }
        Self {
            //clip_rectangles,
            context,
            draw_surface: draw_surface.clone(),
            edit_surface: edit_surface.clone(),
            widget: widget.clone(),
        }
    }
}

impl<W: WidgetExt> Deref for DrawContext<W> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<W: WidgetExt> Drop for DrawContext<W> {
    fn drop(&mut self) {
        self.draw_surface.set(&self.edit_surface);
        self.widget.queue_draw();
        // FIXME: maybe should not call queue_draw() so that the user can only draw a sub-region?
        /*let rects = &self.clip_rectangles.rectangles;
        let window = self.widget.get_window().expect("window");
        use gdk::WindowExt;
        for i in 0..rects.len() {
            let rect = rects.get(i).expect("rectangle");
            let rect = ::gdk::Rectangle {
                x: rect.x as i32,
                y: rect.y as i32,
                width: rect.width as i32,
                height: rect.height as i32,
            };
            window.invalidate_rect(&rect, false);
        }*/
    }
}

/// Manager for drawing operations.
pub struct DrawHandler<W> {
    draw_surface: Surface,
    // FIXME: don't use ImageSurface. It's supposedly slow: https://news.ycombinator.com/item?id=16540587
    edit_surface: ImageSurface,
    widget: Option<W>,
}

impl<W: Clone + WidgetExt> DrawHandler<W> {
    /// Create a new DrawHandler.
    pub fn new() -> Result<Self, cairo::Status> {
        Ok(Self {
            draw_surface: Surface::new(ImageSurface::create(Format::ARgb32, 100, 100)?),
            edit_surface: ImageSurface::create(Format::ARgb32, 100, 100)?,
            widget: None,
        })
    }

    /// Get the drawing context to draw on a widget.
    pub fn get_context(&mut self, clip_rectangles: &[Rectangle]) -> DrawContext<W> {
        if let Some(ref widget) = self.widget {
            let allocation = widget.get_allocation();
            let width = allocation.width;
            let height = allocation.height;
            if (width, height) != (self.edit_surface.get_width(), self.edit_surface.get_height()) {
                // TODO: also copy the old small surface to the new bigger one?
                match ImageSurface::create(Format::ARgb32, width, height) {
                    Ok(surface) => self.edit_surface = surface,
                    Err(error) => eprintln!("Cannot resize image surface: {:?}", error),
                }
            }
            DrawContext::new(&self.draw_surface, &self.edit_surface, widget, clip_rectangles)
        }
        else {
            panic!("Call DrawHandler::init() before DrawHandler::get_context().");
        }
    }

    /// Initialize the draw handler.
    /// The widget is the one on which drawing will occur.
    pub fn init(&mut self, widget: &W/*, stream: &EventStream<MSG>, msg: CALLBACK*/)
    /*where CALLBACK: Fn(RectangleVec) -> MSG + 'static,
          MSG: 'static,*/
    {
        widget.set_app_paintable(true);
        //widget.set_double_buffered(false);
        self.widget = Some(widget.clone());
        let draw_surface = self.draw_surface.clone();
        //let stream = stream.clone();
        widget.connect_draw(move |_, context| {
            // TODO: only copy the area that was exposed?
            context.set_source_surface(&draw_surface.get(), 0.0, 0.0);
            context.paint();
            //stream.emit(msg(context.copy_clip_rectangle_list()));
            /*if clip_rectangles.borrow().is_none() {
                // FIXME: it seems the clip rectangles are out of sync between here and where the
                // actual drawing happens.
                // FIXME: Do not store the rectangles in the DrawHandler, because they will be overriden. Instead, send them in a Draw message.
                *clip_rectangles.borrow_mut() = Some(context.copy_clip_rectangle_list());
            }*/
            Inhibit(false)
        });
    }
}

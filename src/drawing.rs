//! Utility to help drawing on a widget in a relm application.
//! Create a DrawHandler, initialize it, and get its context when handling a message (that could be
//! sent from the draw signal).

// TODO: check if clip has the intended behavior.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use cairo::{
    Context,
    Format,
    ImageSurface,
};
use gtk::{
    Inhibit,
    prelude::WidgetExt,
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
    context: Context,
    draw_surface: Surface,
    edit_surface: ImageSurface,
    widget: W,
}

impl<W: Clone + WidgetExt> DrawContext<W> {
    fn new(draw_surface: &Surface, edit_surface: &ImageSurface, widget: &W) -> Result<Self, cairo::Error> {
        let draw_context = Self {
            context: Context::new(edit_surface)?,
            draw_surface: draw_surface.clone(),
            edit_surface: edit_surface.clone(),
            widget: widget.clone(),
        };

        Ok(draw_context)
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
    }
}

/// Manager for drawing operations.
pub struct DrawHandler<W> {
    draw_surface: Surface,
    edit_surface: ImageSurface,
    widget: Option<W>,
}

impl<W: Clone + WidgetExt> DrawHandler<W> {
    /// Create a new DrawHandler.
    pub fn new() -> Result<Self, cairo::Error> {
        Ok(Self {
            draw_surface: Surface::new(ImageSurface::create(Format::ARgb32, 100, 100)?),
            edit_surface: ImageSurface::create(Format::ARgb32, 100, 100)?,
            widget: None,
        })
    }

    /// Get the drawing context to draw on a widget.
    pub fn get_context(&mut self) -> Result<DrawContext<W>, cairo::Error> {
        if let Some(ref widget) = self.widget {
            let allocation = widget.allocation();
            let scale = if cfg!(feature = "hidpi") {
                widget.scale_factor()
            } else {
                1
            };
            let width = allocation.width() * scale;
            let height = allocation.height() * scale;
            if (width, height) != (self.edit_surface.width(), self.edit_surface.height()) {
                // TODO: also copy the old small surface to the new bigger one?
                match ImageSurface::create(Format::ARgb32, width, height) {
                    Ok(surface) => {
                        {
                            #[cfg(feature = "hidpi")]
                            surface.set_device_scale(scale as f64, scale as f64);
                        }
                        self.edit_surface = surface
                    }
                    Err(error) => eprintln!("Cannot resize image surface: {:?}", error),
                }
            }
            DrawContext::new(&self.draw_surface, &self.edit_surface, widget)
        }
        else {
            panic!("Call DrawHandler::init() before DrawHandler::get_context().");
        }
    }

    /// Initialize the draw handler.
    /// The widget is the one on which drawing will occur.
    pub fn init(&mut self, widget: &W) {
        widget.set_app_paintable(true);
        self.widget = Some(widget.clone());
        let draw_surface = self.draw_surface.clone();
        widget.connect_draw(move |_, context| {
            // TODO: only copy the area that was exposed?
            if let Err(error) = context.set_source_surface(&draw_surface.get(), 0.0, 0.0) {
                eprintln!("Cannot set source surface: {:?}", error);
            }

            if let Err(error) = context.paint() {
                eprintln!("Cannot paint: {:?}", error);
            }

            Inhibit(false)
        });
    }
}

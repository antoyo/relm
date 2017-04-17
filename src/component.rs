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

use std::sync::{Arc, Mutex};

use cairo;
use glib::object::{GObject, ObjectRef};
use glib::translate::{Stash, ToGlibPtr};
use glib::wrapper::{UnsafeFrom, Wrapper};
use gtk;
use gtk::{
    Container,
    ContainerExt,
    IsA,
    Object,
    StaticType,
    Type,
    WidgetExt,
};
use gtk_sys;

use super::{ContainerWidget, DisplayVariant, EventStream, Receiver, RemoteRelm, Widget, create_widget, init_component};

#[derive(Clone)]
pub struct Comp<WIDGET: Widget> {
    pub model: Arc<Mutex<WIDGET::Model>>,
    pub _receiver: Arc<Receiver>,
    pub stream: EventStream<WIDGET::Msg>,
    pub widget: WIDGET,
}

impl<WIDGET: Widget> Drop for Comp<WIDGET> {
    fn drop(&mut self) {
        let _ = self.stream.close();
    }
}

/// Widget that was added by the `ContainerWidget::add_widget()` method.
///
/// ## Warning
/// You must keep your components as long as you want them to send/receive events.
/// Common practice is to store `Component`s in the `Widget` struct (see the [communication
/// example](https://github.com/antoyo/relm/blob/master/examples/communication.rs#L182-L188)).
/// The `#[widget]` attribute takes care of storing them in the struct automatically (see the
/// [communication-attribute example](https://github.com/antoyo/relm/blob/master/examples/communication-attribute.rs)).
#[must_use]
#[derive(Clone)]
pub struct Component<WIDGET: Widget>(Comp<WIDGET>)
    where WIDGET::Model: Clone;

impl<WIDGET: Widget> Component<WIDGET>
    where WIDGET::Model: Clone
{
    #[doc(hidden)]
    pub fn new(component: Comp<WIDGET>) -> Self {
        Component(component)
    }
}

impl<WIDGET: Widget> Component<WIDGET>
    where WIDGET::Model: Clone
{
    /// Get the event stream of the widget.
    /// This is used internally by the library.
    pub fn stream(&self) -> &EventStream<WIDGET::Msg> {
        &self.0.stream
    }

    /// Get the widget of this component.
    pub fn widget(&self) -> &WIDGET {
        &self.0.widget
    }
}

impl<WIDGET: Widget> ContainerExt for Component<WIDGET>
    where WIDGET::Container: ContainerExt,
          WIDGET::Model: Clone,
{
    fn add<T: IsA<gtk::Widget>>(&self, widget: &T) { self.0.widget.container().add(widget) }
    fn check_resize(&self) { self.0.widget.container().check_resize() }
    fn child_notify<T: IsA<gtk::Widget>>(&self, child: &T, child_property: &str) { self.0.widget.container().child_notify(child, child_property) }
    fn child_type(&self) -> Type { self.0.widget.container().child_type() }
    fn get_border_width(&self) -> u32 { self.0.widget.container().get_border_width() }
    fn get_children(&self) -> Vec<gtk::Widget> { self.0.widget.container().get_children() }
    fn get_focus_child(&self) -> Option<gtk::Widget> { self.0.widget.container().get_focus_child() }
    fn get_focus_hadjustment(&self) -> Option<gtk::Adjustment> { self.0.widget.container().get_focus_hadjustment() }
    fn get_focus_vadjustment(&self) -> Option<gtk::Adjustment> { self.0.widget.container().get_focus_vadjustment() }
    fn get_resize_mode(&self) -> gtk::ResizeMode { self.0.widget.container().get_resize_mode() }
    fn propagate_draw<T: IsA<gtk::Widget>>(&self, child: &T, cr: &cairo::Context) { self.0.widget.container().propagate_draw(child, cr) }
    fn remove<T: IsA<gtk::Widget>>(&self, widget: &T) { self.0.widget.container().remove(widget) }
    fn resize_children(&self) { self.0.widget.container().resize_children() }
    fn set_border_width(&self, border_width: u32) { self.0.widget.container().set_border_width(border_width) }
    fn set_focus_chain(&self, focusable_widgets: &[gtk::Widget]) { self.0.widget.container().set_focus_chain(focusable_widgets) }
    fn set_focus_child<T: IsA<gtk::Widget>>(&self, child: Option<&T>) { self.0.widget.container().set_focus_child(child) }
    fn set_focus_hadjustment(&self, adjustment: &gtk::Adjustment) { self.0.widget.container().set_focus_hadjustment(adjustment) }
    fn set_focus_vadjustment(&self, adjustment: &gtk::Adjustment) { self.0.widget.container().set_focus_vadjustment(adjustment) }
    fn set_reallocate_redraws(&self, needs_redraws: bool) { self.0.widget.container().set_reallocate_redraws(needs_redraws) }
    fn set_resize_mode(&self, resize_mode: gtk::ResizeMode) { self.0.widget.container().set_resize_mode(resize_mode) }
    fn unset_focus_chain(&self) { self.0.widget.container().unset_focus_chain() }
    fn set_property_child(&self, child: Option<&gtk::Widget>) { self.0.widget.container().set_property_child(child) }
    fn connect_add<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_check_resize<F: Fn(&Self) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_remove<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
    fn connect_set_focus_child<F: Fn(&Self, &gtk::Widget) + 'static>(&self, _f: F) -> u64 { unimplemented!() }
}

impl<WIDGET> From<Component<WIDGET>> for ObjectRef
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          WIDGET::Model: Clone,
          ObjectRef: From<WIDGET>,
{
    fn from(_value: Component<WIDGET>) -> Self {
        unimplemented!()
    }
}

impl<'a, WIDGET> ToGlibPtr<'a, *mut gtk_sys::GtkWidget> for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container> + ToGlibPtr<'a, *mut gtk_sys::GtkWidget>,
          WIDGET::Model: Clone,
{
    type Storage = <WIDGET::Container as ToGlibPtr<'a, *mut <gtk::Widget as Wrapper>::GlibType>>::Storage;

    fn to_glib_none(&'a self) -> Stash<'a, *mut <gtk::Widget as Wrapper>::GlibType, Self> {
        unimplemented!()
    }
}

impl<'a, WIDGET> ToGlibPtr<'a, *mut GObject> for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container> + ToGlibPtr<'a, *mut GObject>,
          WIDGET::Model: Clone,
{
    type Storage = <WIDGET::Container as ToGlibPtr<'a, *mut <Object as Wrapper>::GlibType>>::Storage;

    fn to_glib_none(&'a self) -> Stash<'a, *mut <Object as Wrapper>::GlibType, Self> {
        unimplemented!()
    }
}

impl<WIDGET> UnsafeFrom<ObjectRef> for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          WIDGET::Model: Clone,
{
    unsafe fn from(_t: ObjectRef) -> Self {
        unimplemented!()
    }
}

impl<WIDGET> StaticType for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          WIDGET::Model: Clone,
{
    fn static_type() -> Type {
        <WIDGET::Container as StaticType>::static_type()
    }
}

impl<WIDGET> Wrapper for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          WIDGET::Model: Clone,
{
    type GlibType = <WIDGET::Container as Wrapper>::GlibType;
}

impl<WIDGET> IsA<gtk::Widget> for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          for<'a> <WIDGET as Widget>::Container: ToGlibPtr<'a, *mut gtk_sys::GtkWidget>,
          WIDGET::Model: Clone,
          ObjectRef: From<WIDGET>,
{
}

impl<WIDGET> IsA<Object> for Component<WIDGET>
    where WIDGET: Widget,
          WIDGET::Container: ContainerExt + IsA<Container>,
          for<'a> <WIDGET as Widget>::Container: ToGlibPtr<'a, *mut GObject>,
          WIDGET::Model: Clone,
          ObjectRef: From<WIDGET>,
{
}

impl<W: Clone + ContainerExt + IsA<gtk::Widget> + IsA<Object>> ContainerWidget for W {
    fn add_widget<CHILDWIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<CHILDWIDGET>
        where MSG: Clone + DisplayVariant + Send + 'static,
              CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Container: IsA<Object> + WidgetExt,
              CHILDWIDGET::Model: Clone + Send,
              CHILDWIDGET::Msg: Clone + DisplayVariant + Send + 'static,
    {
        let component = create_widget::<CHILDWIDGET>(&relm.remote);
        self.add(component.widget.container());
        component.widget.on_add(self.clone());
        init_component::<CHILDWIDGET>(&component, &relm.remote);
        Component(component)
    }

    fn remove_widget<WIDGET>(&self, component: Component<WIDGET>)
        where WIDGET: Widget,
              WIDGET::Container: IsA<gtk::Widget>,
              WIDGET::Model: Clone,
    {
        self.remove(component.0.widget.container());
    }
}

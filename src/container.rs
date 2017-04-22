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

use gtk;
use gtk::{ContainerExt, IsA, Object, WidgetExt};

use component::Component;
use super::{DisplayVariant, RemoteRelm, create_widget, init_component};
use widget::Widget;

/// Trait to implement relm container widget.
pub trait Container: Widget {
    /// The type of the containing widget, i.e. where the child widgets will be added.
    type Container;

    /// Get the containing widget, i.e. the widget where the children will be added.
    fn container(&self) -> &Self::Container;
}

/// Extension trait for GTK+ containers to add and remove relm `Widget`s.
pub trait ContainerWidget {
    /// Add a relm `Widget` to the current GTK+ container.
    ///
    /// # Note
    ///
    /// The returned `Component` must be stored in a `Widget`. If it is not stored, a communication
    /// receiver will be droped which will cause events to be ignored for this widget.
    fn add_widget<WIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<WIDGET>
        where MSG: Clone + DisplayVariant + Send + 'static,
              WIDGET: Widget + 'static,
              WIDGET::Model: Clone + Send,
              WIDGET::Msg: Clone + DisplayVariant + Send + 'static,
              WIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt;

    /// Remove a relm `Widget` from the current GTK+ container.
    fn remove_widget<WIDGET>(&self, component: Component<WIDGET>)
        where WIDGET: Widget,
              WIDGET::Model: Clone,
              WIDGET::Root: IsA<gtk::Widget>;
}

impl<W: Clone + ContainerExt + IsA<gtk::Widget> + IsA<Object>> ContainerWidget for W {
    fn add_widget<CHILDWIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<CHILDWIDGET>
        where MSG: Clone + DisplayVariant + Send + 'static,
              CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Model: Clone + Send,
              CHILDWIDGET::Msg: Clone + DisplayVariant + Send + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt,
    {
        let component = create_widget::<CHILDWIDGET>(&relm.remote);
        self.add(component.widget.root());
        component.widget.on_add(self.clone());
        init_component::<CHILDWIDGET>(&component, &relm.remote);
        Component::new(component)
    }

    fn remove_widget<WIDGET>(&self, component: Component<WIDGET>)
        where WIDGET: Widget,
              WIDGET::Model: Clone,
              WIDGET::Root: IsA<gtk::Widget>,
    {
        self.remove(component.widget().root());
    }
}

/// Trait for relm containers to add GTK+ and relm `Widget`s.
pub trait RelmContainer {
    /// Add a GTK+ widget to a relm container.
    fn add<W: IsA<gtk::Widget>>(&self, widget: &W);

    /// Add a relm widget to a relm container.
    fn add_widget<CHILDWIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<CHILDWIDGET>
        where MSG: Clone + DisplayVariant,
              CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Model: Clone + Send,
              CHILDWIDGET::Msg: Send;

    // TODO: add delete methods?
}

impl<WIDGET> RelmContainer for Component<WIDGET>
    where WIDGET: Container + Widget,
          WIDGET::Container: Clone + ContainerExt + IsA<gtk::Widget> + IsA<Object>,
          WIDGET::Model: Clone,
{
    fn add<W: IsA<gtk::Widget>>(&self, widget: &W) {
        let container = self.widget().container();
        container.add(widget);
    }

    fn add_widget<CHILDWIDGET, MSG>(&self, relm: &RemoteRelm<MSG>) -> Component<CHILDWIDGET>
        where MSG: Clone + DisplayVariant,
              CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Model: Clone + Send,
              CHILDWIDGET::Msg: Send,
    {
        let component = create_widget::<CHILDWIDGET>(&relm.remote);
        let container = self.widget().container();
        container.add(component.widget.root());
        component.widget.on_add(container.clone());
        init_component::<CHILDWIDGET>(&component, &relm.remote);
        Component::new(component)
    }
}

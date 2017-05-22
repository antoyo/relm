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
use gtk::{Cast, ContainerExt, IsA, Object, WidgetExt};

use relm_state::Component;
use super::{DisplayVariant, Relm, create_widget, init_component};
use widget::Widget;

/// Trait to implement relm container widget.
pub trait Container: Widget {
    /// The type of the containing widget, i.e. where the child widgets will be added.
    type Container: Clone + IsA<gtk::Container> + IsA<Object>;

    /// Get the containing widget, i.e. the widget where the children will be added.
    fn container(&self) -> &Self::Container;

    /// Add a GTK+ widget to this container.
    fn add<W: IsA<gtk::Widget>>(&self, widget: &W) {
        self.container().add(widget);
    }

    /// Add a relm widget to this container.
    fn add_widget<WIDGET: Widget>(&self, widget: &WIDGET) -> gtk::Container {
        let container = self.container();
        container.add(&widget.root());
        container.clone().upcast()
    }
}

/// Extension trait for GTK+ containers to add and remove relm `Widget`s.
pub trait ContainerWidget {
    /// Add a relm `Widget` to the current GTK+ container.
    ///
    /// # Note
    ///
    /// The returned `Component` must be stored in a `Widget`. If it is not stored, a communication
    /// receiver will be droped which will cause events to be ignored for this widget.
    fn add_widget<CHILDWIDGET, WIDGET>(&self, relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt,
              WIDGET: Widget;

    /// Remove a relm `Widget` from the current GTK+ container.
    fn remove_widget<CHILDWIDGET>(&self, component: Component<CHILDWIDGET>)
        where CHILDWIDGET: Widget,
              CHILDWIDGET::Root: IsA<gtk::Widget>;
}

impl<W: Clone + ContainerExt + IsA<gtk::Widget> + IsA<Object>> ContainerWidget for W {
    fn add_widget<CHILDWIDGET, WIDGET>(&self, relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt,
    {
        let (component, child_relm) = create_widget::<CHILDWIDGET>(relm.context(), model_param);
        {
            let widget = component.widget();
            self.add(&widget.root());
            widget.on_add(self.clone());
        }
        init_component::<CHILDWIDGET>(&component, relm.context(), &child_relm);
        component
    }

    fn remove_widget<WIDGET>(&self, component: Component<WIDGET>)
        where WIDGET: Widget,
              WIDGET::Root: IsA<gtk::Widget>,
    {
        self.remove(&component.widget().root());
    }
}

/// Trait for relm containers to add GTK+ and relm `Widget`s.
pub trait RelmContainer {
    /// Add a GTK+ widget to a relm container.
    fn add<W: IsA<gtk::Widget>>(&self, widget: &W);

    /// Add a relm widget to a relm container.
    fn add_widget<CHILDWIDGET, WIDGET>(&self, relm: &Relm<WIDGET>, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              WIDGET: Widget;

    // TODO: add delete methods?
}

impl<WIDGET> RelmContainer for Component<WIDGET>
    where WIDGET: Container + Widget,
          WIDGET::Container: ContainerExt + IsA<gtk::Widget> + IsA<Object>,
{
    fn add<W: IsA<gtk::Widget>>(&self, widget: &W) {
        self.widget().add(widget);
    }

    fn add_widget<CHILDWIDGET, PARENTWIDGET>(&self, relm: &Relm<PARENTWIDGET>, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              PARENTWIDGET: Widget
    {
        let (component, child_relm) = create_widget::<CHILDWIDGET>(relm.context(), model_param);
        {
            let widget = component.widget();
            let container = self.widget().add_widget(&*widget);
            widget.on_add(container.clone());
        }
        init_component::<CHILDWIDGET>(&component, relm.context(), &child_relm);
        component
    }
}

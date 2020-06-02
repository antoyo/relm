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

use glib::{Cast, IsA, Object};
use gtk::{ContainerExt, WidgetExt};

use crate::state::EventStream;
use super::{Component, DisplayVariant, create_widget, init_component};
use crate::widget::Widget;

/// Struct for relm containers to add GTK+ and relm `Widget`s.
pub struct ContainerComponent<WIDGET: Container + Widget> {
    component: Component<WIDGET>,
    /// The default container of this component.
    pub container: WIDGET::Container,
    /// Additional containers used for multi-containers. This can be () if not needed.
    pub containers: WIDGET::Containers,
}

/*impl<WIDGET: Container + Widget> Clone for ContainerComponent<WIDGET> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            container: self.container.clone(),
            containers: self.containers.clone(),
        }
    }
}*/

impl<WIDGET: Container + Widget> ContainerComponent<WIDGET> {
    #[doc(hidden)]
    pub fn new(component: Component<WIDGET>, container: WIDGET::Container, containers: WIDGET::Containers) -> Self {
        ContainerComponent {
            component,
            container,
            containers,
        }
    }

    /// Add a GTK+ widget to a relm container.
    pub fn add<CHILDWIDGET: IsA<gtk::Widget>>(&self, widget: &CHILDWIDGET) {
        self.container.add(widget);
    }

    /// Add a relm widget to a relm container.
    pub fn add_widget<CHILDWIDGET>(&self, model_param: CHILDWIDGET::ModelParam)
        -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              WIDGET::Container: ContainerExt + IsA<gtk::Widget> + IsA<Object>,
    {
        let (component, widget, child_relm) = create_widget::<CHILDWIDGET>(model_param);
        let container = WIDGET::add_widget(self, &component);
        widget.on_add(container);
        init_component::<CHILDWIDGET>(component.owned_stream(), widget, &child_relm);
        component
    }

    /// Emit a message of the widget stream.
    pub fn emit(&self, msg: WIDGET::Msg) {
        self.owned_stream().emit(msg);
    }

    /// Get the event stream of the component.
    /// This is used internally by the library.
    pub fn owned_stream(&self) -> &EventStream<WIDGET::Msg> {
        self.component.owned_stream()
    }

    // TODO: add delete methods?

    /// Get the widget of the component.
    pub fn widget(&self) -> &WIDGET::Root {
        self.component.widget()
    }
}

/// Trait to implement relm container widget.
pub trait Container: Widget {
    /// The type of the containing widget, i.e. where the child widgets will be added.
    type Container: Clone + IsA<gtk::Container> + IsA<Object> + IsA<gtk::Widget>;
    /// Type to contain the additional container widgets.
    // TODO: put that in yet another trait?
    type Containers: Clone;

    /// Add a relm widget to this container.
    /// Return the widget that will be send to Widget::on_add().
    fn add_widget<WIDGET: Widget>(container: &ContainerComponent<Self>, component: &Component<WIDGET>)
        -> gtk::Container
    {
        container.container.add(component.widget());
        container.container.clone().upcast()
    }

    /// Get the containing widget, i.e. the widget where the children will be added.
    fn container(&self) -> &Self::Container;

    /// Get additional container widgets.
    /// This is useful to create a multi-container.
    fn other_containers(&self) -> Self::Containers;
}

/// Extension trait for GTK+ containers to add and remove relm `Widget`s.
pub trait ContainerWidget {
    /// Add a relm `Container` to the current GTK+ container.
    ///
    /// # Note
    ///
    /// The returned `ContainerComponent` must be stored in a `Widget`. If it is not stored, a
    /// communication receiver will be droped which will cause events to be ignored for this
    /// widget.
    fn add_container<CHILDWIDGET>(&self, model_param: CHILDWIDGET::ModelParam)
            -> ContainerComponent<CHILDWIDGET>
        where CHILDWIDGET: Container + Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt;

    /// Add a relm `Widget` to the current GTK+ container.
    ///
    /// # Note
    ///
    /// The returned `Component` must be stored in a `Widget`. If it is not stored, a communication
    /// receiver will be droped which will cause events to be ignored for this widget.
    fn add_widget<CHILDWIDGET>(&self, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt;

    /// Remove a relm `Widget` from the current GTK+ container.
    fn remove_widget<CHILDWIDGET>(&self, component: Component<CHILDWIDGET>)
        where CHILDWIDGET: Widget,
              CHILDWIDGET::Root: IsA<gtk::Widget>;
}

impl<W: Clone + ContainerExt + IsA<gtk::Widget> + IsA<Object>> ContainerWidget for W {
    fn add_container<CHILDWIDGET>(&self, model_param: CHILDWIDGET::ModelParam)
            -> ContainerComponent<CHILDWIDGET>
        where CHILDWIDGET: Container + Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt,
    {
        let (component, widget, child_relm) = create_widget::<CHILDWIDGET>(model_param);
        let container = widget.container().clone();
        let containers = widget.other_containers();
        let root = widget.root().clone();
        self.add(&root);
        widget.on_add(self.clone());
        init_component::<CHILDWIDGET>(component.owned_stream(), widget, &child_relm);
        ContainerComponent::new(component, container, containers)
    }

    fn add_widget<CHILDWIDGET>(&self, model_param: CHILDWIDGET::ModelParam)
            -> Component<CHILDWIDGET>
        where CHILDWIDGET: Widget + 'static,
              CHILDWIDGET::Msg: DisplayVariant + 'static,
              CHILDWIDGET::Root: IsA<gtk::Widget> + IsA<Object> + WidgetExt,
    {
        let (component, widget, child_relm) = create_widget::<CHILDWIDGET>(model_param);
        self.add(component.widget());
        widget.on_add(self.clone());
        init_component::<CHILDWIDGET>(component.owned_stream(), widget, &child_relm);
        component
    }

    // TODO: we're probably not calling remove_widget() when removing a relm widget from a gtk
    // widget.
    fn remove_widget<WIDGET>(&self, component: Component<WIDGET>)
        where WIDGET: Widget,
              WIDGET::Root: IsA<gtk::Widget>,
    {
        self.remove(component.widget());
    }
}

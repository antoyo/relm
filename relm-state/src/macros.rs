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

#[macro_export]
#[doc(hidden)]
macro_rules! check_recursion {
    ($widget:ident) => {
        if $widget.try_borrow_mut().is_err() {
            panic!("An event to the same widget was emitted in the update() method, which would cause an infinite \
                   recursion.\nThis can be caused by calling a gtk+ function that is connected to send a message \
                   to the same widget.\nInspect the stack trace to determine which call it is.\nThen you can either \
                   refactor your code to avoid a cyclic event dependency or block events from being emitted by doing \
                   the following:\n{\n    let _lock = self.model.relm.stream().lock();\n    // Your panicking call.\n}\
                   \nSee this example: \
                   https://github.com/antoyo/relm/blob/feature/futures-glib/examples/checkboxes.rs#L88\
                   This issue can also happen when emitting a signal to the same widget, in which case you need to\
                   refactor your code to avoid this cyclic event dependency.");
        }
    };
}

/// Connect events to sending a message.
///
/// Rule #1:
/// Send `$msg` to `$other_component` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #2:
/// Optionally send `$msg.0` when the GTK+ `$event` is emitted on `$widget`.
/// Return `$msg.1` in the GTK+ callback.
/// This variant gives more control to the caller since it expects a `$msg` returning `(Option<MSG>,
/// ReturnValue)` where the `ReturnValue` is the value to return in the GTK+ callback.
/// Option<MSG> can be None if no message needs to be emitted.
///
/// Rule #3:
/// Send `$msg` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #4:
/// Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_component:expr, $msg:expr) => {
        let stream = $other_component.stream().clone();
        $widget.$event(move |$($args),*| {
            let msg: Option<_> = ::relm::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    };

    // Connect to a GTK+ widget event.
    // This variant gives more control to the caller since it expects a `$msg` returning (Option<MSG>,
    // ReturnValue) where the ReturnValue is the value to return in the GTK+ callback.
    // Option<MSG> can be None if no message needs to be emitted.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), return $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let (msg, return_value) = ::relm::IntoPair::into_pair($msg);
            let msg: Option<_> = ::relm::IntoOption::into_option(msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            return_value
        });
    }};

    // Connect to a GTK+ widget event.
    // This variant gives more control to the caller since it expects a `$msg` returning (Option<MSG>,
    // ReturnValue) where the ReturnValue is the value to return in the GTK+ callback.
    // Option<MSG> can be None if no message needs to be emitted.
    // This variant also give you a widget so that you can call a function that will use and mutate
    // its model.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), with return $widget_clone:ident $msg:expr) => {{
        let stream = $relm.stream().clone();
        #[allow(unused_mut)]
        $widget.$event(move |$($args),*| {
            let $widget_clone = $widget_clone.upgrade().expect("upgrade should always work");
            check_recursion!($widget_clone);
            let mut $widget_clone = $widget_clone.borrow_mut();
            let (msg, return_value) = ::relm::IntoPair::into_pair($msg);
            let msg: Option<_> = ::relm::IntoOption::into_option(msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            return_value
        });
    }};

    // Connect to a GTK+ widget event.
    // This variant allows to call a method that will return the message
    // Option<MSG> can be None if no message needs to be emitted.
    // This variant also give you a widget so that you can call a function that will use and mutate
    // its model.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), with $widget_clone:ident $msg:expr) => {{
        let stream = $relm.stream().clone();
        #[allow(unused_mut)]
        $widget.$event(move |$($args),*| {
            let $widget_clone = $widget_clone.upgrade().expect("upgrade should always work");
            check_recursion!($widget_clone);
            let mut $widget_clone = $widget_clone.borrow_mut();
            let msg: Option<_> = ::relm::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    }};

    // Connect to a message reception.
    // This variant also give you a widget so that you can call a function that will use and mutate
    // its model.
    ($src_component:ident @ $message:pat, $dst_component:ident, with $widget:ident $msg:expr) => {
        let stream = $dst_component.stream().clone();
        $src_component.stream().observe(move |msg| {
            #[allow(unreachable_patterns, unused_mut)]
            match msg {
                &$message =>  {
                    let $widget = $widget.upgrade().expect("upgrade should always work");
                    check_recursion!($widget);
                    let mut $widget = $widget.borrow_mut();
                    let msg: Option<_> = ::relm::IntoOption::into_option($msg);
                    if let Some(msg) = msg {
                        stream.emit(msg);
                    }
                },
                _ => (),
            }
        });
    };

    // Connect to a GTK+ widget event where the return value is retrieved asynchronously.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), async $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let (resolver, rx) = ::relm::Resolver::channel();
            let msg: Option<_> = ::relm::IntoOption::into_option($msg(resolver));
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            ::gtk::main();
            // TODO: remove unwrap().
            ::futures::Stream::wait(rx).next().unwrap().unwrap()
        });
    }};

    // Connect to a GTK+ widget event.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let msg: Option<_> = ::relm::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    }};

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($src_component:ident @ $message:pat, $dst_component:ident, $msg:expr) => {
        let stream = $dst_component.stream().clone();
        $src_component.stream().observe(move |msg| {
            #[allow(unreachable_patterns)]
            match msg {
                &$message =>  {
                    let msg: Option<_> = ::relm::IntoOption::into_option($msg);
                    if let Some(msg) = msg {
                        stream.emit(msg);
                    }
                },
                _ => (),
            }
        });
    };
}

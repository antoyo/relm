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
/// Send `$msg` with a Resolver and block the callback into it resolves to a value.
/// This is useful in case you need an access to the model to decide whether you want to inhibit
/// the event or not.
///
/// Rule #4:
/// Send `$msg` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #5:
/// Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_component:expr, $msg:expr) => {
        connect_stream!($widget, $event($($args),*), $other_component.stream(), $msg);
    };

    // Connect to a GTK+ widget event.
    // This variant gives more control to the caller since it expects a `$msg` returning (Option<MSG>,
    // ReturnValue) where the ReturnValue is the value to return in the GTK+ callback.
    // Option<MSG> can be None if no message needs to be emitted.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), return $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let (msg, return_value) = $crate::IntoPair::into_pair($msg);
            let msg: Option<_> = $crate::IntoOption::into_option(msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            return_value
        });
    }};

    // Connect to a GTK+ widget event where the return value is retrieved asynchronously.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), async $msg:ident $( ( $( $arg:expr ),*) )* ) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let cx = ::futures_glib::MainContext::default(|cx| cx.clone());
            let lp = ::relm::MainLoop::new(Some(&cx));
            let (resolver, rx) = ::relm::Resolver::channel(lp.clone());
            let msg: Option<_> = $crate::IntoOption::into_option($msg($($($arg,)*)* resolver));
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            lp.run();
            // TODO: remove unwrap().
            ::futures::Stream::wait(rx).next().unwrap().unwrap()
        });
    }};

    // Connect to a GTK+ widget event.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            let msg: Option<_> = $crate::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    }};

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($src_component:ident @ $message:pat, $dst_component:expr, $msg:expr) => {
        let stream = $src_component.stream();
        connect_stream!(stream@$message, $dst_component.stream(), $msg);
    };
}

/// Connect events to sending a message.
/// Similar to `connect!` but wants a stream instead of a component.
///
/// Rule #1:
/// Send `$msg` to `$other_stream` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #2:
/// Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect_stream {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_stream:expr, $msg:expr) => {
        let stream = $other_stream.clone();
        $widget.$event(move |$($args),*| {
            let msg: Option<_> = $crate::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    };

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($src_stream:ident @ $message:pat, $dst_stream:expr, $msg:expr) => {
        let stream = $dst_stream.clone();
        $src_stream.observe(move |msg| {
            #[allow(unreachable_patterns)]
            match msg {
                &$message =>  {
                    let msg: Option<_> = $crate::IntoOption::into_option($msg);
                    if let Some(msg) = msg {
                        stream.emit(msg);
                    }
                },
                _ => (),
            }
        });
    };
}

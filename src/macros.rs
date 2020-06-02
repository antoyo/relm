/*
 * Copyright (c) 2017-2019 Boucher, Antoni <bouanto@zoho.com>
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
/// ## Rules
/// 1. Send `$msg` to `$other_component` when the GTK+ `$event` is emitted on `$widget`.
///
/// 2. Optionally send `$msg.0` when the GTK+ `$event` is emitted on `$widget`.
/// Return `$msg.1` in the GTK+ callback.
/// This variant gives more control to the caller since it expects a `$msg` returning `(Option<MSG>,
/// ReturnValue)` where the `ReturnValue` is the value to return in the GTK+ callback.
/// Option<MSG> can be None if no message needs to be emitted.
///
/// 3. Send `$msg` when the GTK+ `$event` is emitted on `$widget`.
///
/// 4. Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_component:expr, $msg:expr) => {
        $crate::connect_stream!($widget, $event($($args),*), $other_component.stream(), $msg);
    };

    // Connect to a GTK+ widget event.
    // This variant gives more control to the caller since it expects a `$msg` returning (Option<MSG>,
    // ReturnValue) where the ReturnValue is the value to return in the GTK+ callback.
    // Option<MSG> can be None if no message needs to be emitted.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), return $msg:expr) => {{
        $crate::connect_stream!(return $relm.stream(), $widget, $event($($args),*), $msg);
    }};

    // Connect to a GTK+ widget event.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $relm.stream().clone();
        let _ = $widget.$event(move |$($args),*| {
            let msg: Option<_> = $crate::IntoOption::into_option($msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
        });
    }};

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($src_component:ident @ $message:pat, $dst_component:expr, $msg:expr) => {
        let stream = $src_component.stream().clone();
        $crate::connect_stream!(stream@$message, $dst_component.stream(), $msg);
    };
}

/// Connect events to sending a message.
/// Similar to `connect!` but wants a stream instead of a component.
///
/// ## Rules
/// 1. Send `$msg` to `$other_stream` when the GTK+ `$event` is emitted on `$widget`.
///
/// 2. Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect_stream {
    // Connect to a GTK+ widget event.
    // This variant gives more control to the caller since it expects a `$msg` returning (Option<MSG>,
    // ReturnValue) where the ReturnValue is the value to return in the GTK+ callback.
    // Option<MSG> can be None if no message needs to be emitted.
    (return $stream:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $stream.clone();
        let _ = $widget.$event(move |$($args),*| {
            let (msg, return_value) = $crate::IntoPair::into_pair($msg);
            let msg: Option<_> = $crate::IntoOption::into_option(msg);
            if let Some(msg) = msg {
                stream.emit(msg);
            }
            return_value
        });
    }};

    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_stream:expr, $msg:expr) => {
        let stream = $other_stream.clone();
        let _ = $widget.$event(move |$($args),*| {
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

/// Connect an asynchronous method call to send a message.
/// The variants with `$fail_msg` will send this message when there's an error.
/// Those without this argument will ignore the error.
#[macro_export]
macro_rules! connect_async {
    ($object:expr, $async_method:ident, $relm:expr, $msg:expr) => {
        connect_async!($object, $async_method(), $relm, $msg)
    };
    ($object:expr, $async_method:ident ( $($args:expr),* ), $relm:expr, $msg:expr) => {{
        // TODO: remove any use of Fragile when gio callbacks stop requiring Send.
        let stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $object.$async_method($($args,)* None::<&gio::Cancellable>, move |result| {
            if let Ok(result) = result {
                stream.into_inner().emit($msg(result));
            }
        });
    }};
    ($object:expr, $async_method:ident, $relm:expr, $msg:expr, $fail_msg:expr) => {
        connect_async!($object, $async_method(), $relm, $msg, $fail_msg)
    };
    ($object:expr, $async_method:ident ( $($args:expr),* ), $relm:expr, $msg:expr, $fail_msg:expr) => {{
        let event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        let fail_event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $object.$async_method($($args,)* None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(value) => event_stream.into_inner().emit($msg(value)),
                Err(error) => fail_event_stream.into_inner().emit($fail_msg(error)),
            }
        });
    }};
}

/// Connect an asynchronous function call to send a message.
/// The variants with `$fail_msg` will send this message when there's an error.
/// Those without this argument will ignore the error.
#[macro_export]
macro_rules! connect_async_func {
    ($class:ident :: $async_function:ident, $relm:expr, $msg:expr) => {
        connect_async_func!($class::$async_func(), $relm, $msg)
    };
    ($class:ident :: $async_func:ident ( $($args:expr),* ), $relm:expr, $msg:expr) => {{
        let stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $class::$async_func($($args,)* None::<&gio::Cancellable>, move |result| {
            if let Ok(result) = result {
                stream.into_inner().emit($msg(result));
            }
        });
    }};
    ($class:ident :: $async_func:ident, $relm:expr, $msg:expr, $fail_msg:expr) => {
        connect_async_func!($class::$async_func(), $relm, $msg, $fail_msg)
    };
    ($class:ident :: $async_func:ident ( $($args:expr),* ), $relm:expr, $msg:expr, $fail_msg:expr) => {{
        let event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        let fail_event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $class::$async_func($($args,)* None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(value) => event_stream.into_inner().emit($msg(value)),
                Err(error) => fail_event_stream.into_inner().emit($fail_msg(error)),
            }
        });

    }};
}

/// Like `connect_async!`, but also return a `Cancellable` to control the asynchronous request.
#[macro_export]
macro_rules! connect_async_full {
    ($object:expr, $async_method:ident, $relm:expr, $msg:expr) => {
        connect_async_full!($object, $async_method(), $relm, $msg)
    };
    ($object:expr, $async_method:ident ( $($args:expr),* ), $relm:expr, $msg:expr) => {{
        let cancellable = ::gio::Cancellable::new();
        let stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $object.$async_method($($args,)* Some(&cancellable), move |result| {
            if let Ok(result) = result {
                stream.into_inner().emit($msg(result));
            }
        });
        cancellable
    }};
    ($object:expr, $async_method:ident, $relm:expr, $msg:expr, $fail_msg:expr) => {
        connect_async_full!($object, $async_method(), $relm, $msg, $fail_msg)
    };
    ($object:expr, $async_method:ident ( $($args:expr),* ), $relm:expr, $msg:expr, $fail_msg:expr) => {{
        let cancellable = ::gio::Cancellable::new();
        let event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        let fail_event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $object.$async_method($($args,)* Some(&cancellable), move |result| {
            match result {
                Ok(value) => event_stream.into_inner().emit($msg(value)),
                Err(error) => fail_event_stream.into_inner().emit($fail_msg(error)),
            }
        });

        cancellable
    }};
}

/// Like `connect_async_func!`, but also return a `Cancellable` to control the asynchronous request.
#[macro_export]
macro_rules! connect_async_func_full {
    ($class:ident :: $async_function:ident, $relm:expr, $msg:expr) => {
        connect_async!($async_func(), $relm, $msg)
    };
    ($class:ident :: $async_func:ident ( $($args:expr),* ), $relm:expr, $msg:expr) => {{
        let cancellable = ::gio::Cancellable::new();
        let stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $class::$async_func($($args,)* Some(&cancellable), move |result| {
            if let Ok(result) = result {
                stream.into_inner().emit($msg(result));
            }
        });
        cancellable
    }};
    ($class:ident :: $async_func:ident, $relm:expr, $msg:expr, $fail_msg:expr) => {
        connect_async!($async_func(), $relm, $msg, $fail_msg)
    };
    ($class:ident :: $async_func:ident ( $($args:expr),* ), $relm:expr, $msg:expr, $fail_msg:expr) => {{
        let cancellable = ::gio::Cancellable::new();
        let event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        let fail_event_stream = ::relm::vendor::fragile::Fragile::new($relm.stream().clone());
        $class::$async_func($($args,)* Some(&cancellable), move |result| {
            match result {
                Ok(value) => event_stream.into_inner().emit($msg(value)),
                Err(error) => fail_event_stream.into_inner().emit($fail_msg(error)),
            }
        });

        cancellable
    }};
}

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

/// Rule #1:
/// Send `$msg` to `$other_component` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #2:
/// Send `$msg` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #3:
/// Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_component:expr, $msg:expr) => {
        let stream = $other_component.stream().clone();
        $widget.$event(move |$($args),*| {
            stream.emit($msg);
        });
    };

    // Connect to a GTK+ widget event.
    ($stream:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $stream.clone();
        $widget.$event(move |$($args),*| {
            stream.emit($msg);
        });
    }};

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($src_component:expr, $message:pat, $dst_component:expr, $msg:expr) => {
        let stream = $dst_component.stream().clone();
        $src_component.stream().observe(move |msg| {
            #[allow(unreachable_patterns)]
            match msg {
                $message =>  {
                    stream.emit($msg);
                },
                _ => (),
            }
        });
    };
}

/// Send `$msg` when the `$event` is emitted on `$widget` (without inhibiting the event).
// TODO: add the missing rules from the connect!() macro.
#[macro_export]
macro_rules! connect_no_inhibit {
    ($stream:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $stream.clone();
        $widget.$event(move |$($args),*| {
            stream.emit($msg);
            ::gtk::Inhibit(false)
        });
    }};
}

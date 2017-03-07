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
/// Send `$msg` to `$other_widget` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #2:
/// Send `$msg` when the GTK+ `$event` is emitted on `$widget`.
///
/// Rule #3:
/// Send `$msg` to `$widget` when the `$message` is received on `$stream`.
#[macro_export]
macro_rules! connect {
    // Connect to a GTK+ widget event, sending a message to another widget.
    ($widget:expr, $event:ident($($args:pat),*), $other_widget:expr, $msg:expr) => {
        let widget = $other_widget.clone();
        $widget.$event(move |$($args),*| {
            widget.emit($msg);
        });
    };

    // Connect to a GTK+ widget event.
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            stream.emit($msg);
        });
    }};

    // Connect to a message reception.
    // TODO: create another macro rule accepting multiple patterns.
    ($stream:expr, $message:pat, $widget:expr, $msg:expr) => {
        let widget = $widget.clone();
        $stream.observe(move |msg| {
            #[allow(unreachable_patterns)]
            match msg {
                $message =>  {
                    widget.emit($msg);
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
    ($relm:expr, $widget:expr, $event:ident($($args:pat),*), $msg:expr) => {{
        let stream = $relm.stream().clone();
        $widget.$event(move |$($args),*| {
            stream.emit($msg);
            ::gtk::Inhibit(false)
        });
    }};
}

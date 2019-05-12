/*
 * Copyright (c) 2019 Boucher, Antoni <bouanto@zoho.com>
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

use std::cell::RefCell;
use std::rc::Rc;

use relm::StreamHandle;

use crate::functions::run_loop;

/// Used to wait for a widget's signal.
///
/// It's recommended to use it with the [`observer_new`] macro.
///
/// Example:
///
/// ```
/// extern crate gtk;
/// #[macro_use]
/// extern crate relm_test;
///
/// use gtk::GtkWindowExt;
///
/// # fn main() {
/// gtk::init().expect("initialization failed");
/// let window = gtk::Window::new(gtk::WindowType::Toplevel);
///
/// let observer = observer_new!(window, connect_activate_focus, |_|);
/// window.emit_activate_focus();
/// observer.wait();
/// # }
/// ```
pub struct Observer {
    result: Rc<RefCell<bool>>,
}

impl Observer {
    /// Returns a new observer.
    ///
    /// It's recommended to not use it directly as is but instead to use the [`observer_new`] macro.
    ///
    /// But anyway, here's an example using it as is:
    ///
    /// ```
    /// extern crate gtk;
    /// #[macro_use]
    /// extern crate relm_test;
    ///
    /// use gtk::GtkWindowExt;
    ///
    /// # fn main() {
    /// gtk::init().expect("GTK init failed");
    ///
    /// let window = gtk::Window::new(gtk::WindowType::Toplevel);
    ///
    /// let observer = relm_test::Observer::new();
    /// let inner = observer.get_inner().clone();
    /// window.connect_activate_focus(move |_| {
    ///     *inner.borrow_mut() = true;
    /// });
    ///
    /// window.emit_activate_focus();
    /// observer.wait();
    /// # }
    /// ```
    pub fn new() -> Observer {
        Observer {
            result: Rc::new(RefCell::new(false)),
        }
    }

    /// Returns the inner field. Just don't use it.
    pub fn get_inner(&self) -> &Rc<RefCell<bool>> {
        &self.result
    }

    /// Wait for the signal to be triggered.
    ///
    /// ```
    /// extern crate gtk;
    /// #[macro_use]
    /// extern crate relm_test;
    ///
    /// use gtk::GtkWindowExt;
    ///
    /// # fn main() {
    /// gtk::init().expect("initialization failed");
    /// let window = gtk::Window::new(gtk::WindowType::Toplevel);
    ///
    /// let observer = observer_new!(window, connect_activate_focus, |_|);
    /// window.emit_activate_focus();
    /// observer.wait();
    /// # }
    /// ```
    pub fn wait(&self) {
        loop {
            if let Ok(ref result) = self.result.try_borrow() {
                if **result == true {
                    break
                }
            }
            run_loop();
        }
    }
}

/// Used to wait for a widget's signal.
///
/// It's recommended to use it with the [`relm_observer_new`] macro.
pub struct RelmObserver<MSG> {
    result: Rc<RefCell<Option<MSG>>>,
}

impl<MSG: Clone + 'static> RelmObserver<MSG> {
    /// Returns a new relm observer.
    ///
    /// It's recommended to not use it directly as is but instead to use the [`relm_observer_new`] macro.
    pub fn new<F: Fn(&MSG) -> bool + 'static>(stream: &StreamHandle<MSG>, predicate: F) -> Self {
        let result = Rc::new(RefCell::new(None));
        let res = result.clone();
        stream.observe(move |msg| {
            if predicate(msg) {
                *res.borrow_mut() = Some(msg.clone());
            }
        });
        Self {
            result,
        }
    }

    /// Wait for the message to be triggered.
    pub fn wait(&self) -> MSG {
        loop {
            if let Ok(ref result) = self.result.try_borrow() {
                if result.is_some() {
                    break;
                }
            }
            run_loop();
        }
        self.result.borrow_mut().take()
            .expect("Message to take")
    }
}

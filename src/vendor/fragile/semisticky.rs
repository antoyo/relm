use std::cmp;
use std::fmt;

use super::errors::InvalidThreadAccess;
use super::fragile::Fragile;
use std::mem;
use super::sticky::Sticky;

enum SemiStickyImpl<T> {
    Fragile(Fragile<T>),
    Sticky(Sticky<T>),
}

/// A `SemiSticky<T>` keeps a value T stored in a thread if it has a drop.
///
/// This is a combined version of `Fragile<T>` and `Sticky<T>`.  If the type
/// does not have a drop it will effectively be a `Fragile<T>`, otherwise it
/// will be internally behave like a `Sticky<T>`.
pub struct SemiSticky<T> {
    inner: SemiStickyImpl<T>,
}

impl<T> SemiSticky<T> {
    /// Creates a new `SemiSticky` wrapping a `value`.
    ///
    /// The value that is moved into the `SemiSticky` can be non `Send` and
    /// will be anchored to the thread that created the object.  If the
    /// sticky wrapper type ends up being send from thread to thread
    /// only the original thread can interact with the value.  In case the
    /// value does not have `Drop` it will be stored in the `SemiSticky`
    /// instead.
    pub fn new(value: T) -> Self {
        SemiSticky {
            inner: if mem::needs_drop::<T>() {
                SemiStickyImpl::Sticky(Sticky::new(value))
            } else {
                SemiStickyImpl::Fragile(Fragile::new(value))
            },
        }
    }

    /// Returns `true` if the access is valid.
    ///
    /// This will be `false` if the value was sent to another thread.
    pub fn is_valid(&self) -> bool {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.is_valid(),
            SemiStickyImpl::Sticky(ref inner) => inner.is_valid(),
        }
    }

    /// Consumes the `SemiSticky`, returning the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if called from a different thread than the one where the
    /// original value was created.
    pub fn into_inner(self) -> T {
        match self.inner {
            SemiStickyImpl::Fragile(inner) => inner.into_inner(),
            SemiStickyImpl::Sticky(inner) => inner.into_inner(),
        }
    }

    /// Consumes the `SemiSticky`, returning the wrapped value if successful.
    ///
    /// The wrapped value is returned if this is called from the same thread
    /// as the one where the original value was created, otherwise the
    /// `SemiSticky` is returned as `Err(self)`.
    pub fn try_into_inner(self) -> Result<T, Self> {
        match self.inner {
            SemiStickyImpl::Fragile(inner) => inner.try_into_inner().map_err(|inner| SemiSticky {
                inner: SemiStickyImpl::Fragile(inner),
            }),
            SemiStickyImpl::Sticky(inner) => inner.try_into_inner().map_err(|inner| SemiSticky {
                inner: SemiStickyImpl::Sticky(inner),
            }),
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get`](#method.try_get`).
    pub fn get(&self) -> &T {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.get(),
            SemiStickyImpl::Sticky(ref inner) => inner.get(),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get_mut`](#method.try_get_mut`).
    pub fn get_mut(&mut self) -> &mut T {
        match self.inner {
            SemiStickyImpl::Fragile(ref mut inner) => inner.get_mut(),
            SemiStickyImpl::Sticky(ref mut inner) => inner.get_mut(),
        }
    }

    /// Tries to immutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get(&self) -> Result<&T, InvalidThreadAccess> {
        match self.inner {
            SemiStickyImpl::Fragile(ref inner) => inner.try_get(),
            SemiStickyImpl::Sticky(ref inner) => inner.try_get(),
        }
    }

    /// Tries to mutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get_mut(&mut self) -> Result<&mut T, InvalidThreadAccess> {
        match self.inner {
            SemiStickyImpl::Fragile(ref mut inner) => inner.try_get_mut(),
            SemiStickyImpl::Sticky(ref mut inner) => inner.try_get_mut(),
        }
    }
}

impl<T> From<T> for SemiSticky<T> {
    #[inline]
    fn from(t: T) -> SemiSticky<T> {
        SemiSticky::new(t)
    }
}

impl<T: Clone> Clone for SemiSticky<T> {
    #[inline]
    fn clone(&self) -> SemiSticky<T> {
        SemiSticky::new(self.get().clone())
    }
}

impl<T: Default> Default for SemiSticky<T> {
    #[inline]
    fn default() -> SemiSticky<T> {
        SemiSticky::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for SemiSticky<T> {
    #[inline]
    fn eq(&self, other: &SemiSticky<T>) -> bool {
        *self.get() == *other.get()
    }
}

impl<T: Eq> Eq for SemiSticky<T> {}

impl<T: PartialOrd> PartialOrd for SemiSticky<T> {
    #[inline]
    fn partial_cmp(&self, other: &SemiSticky<T>) -> Option<cmp::Ordering> {
        self.get().partial_cmp(&*other.get())
    }

    #[inline]
    fn lt(&self, other: &SemiSticky<T>) -> bool {
        *self.get() < *other.get()
    }

    #[inline]
    fn le(&self, other: &SemiSticky<T>) -> bool {
        *self.get() <= *other.get()
    }

    #[inline]
    fn gt(&self, other: &SemiSticky<T>) -> bool {
        *self.get() > *other.get()
    }

    #[inline]
    fn ge(&self, other: &SemiSticky<T>) -> bool {
        *self.get() >= *other.get()
    }
}

impl<T: Ord> Ord for SemiSticky<T> {
    #[inline]
    fn cmp(&self, other: &SemiSticky<T>) -> cmp::Ordering {
        self.get().cmp(&*other.get())
    }
}

impl<T: fmt::Display> fmt::Display for SemiSticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self.get(), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for SemiSticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.try_get() {
            Ok(value) => f.debug_struct("SemiSticky").field("value", value).finish(),
            Err(..) => {
                struct InvalidPlaceholder;
                impl fmt::Debug for InvalidPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<invalid thread>")
                    }
                }

                f.debug_struct("SemiSticky")
                    .field("value", &InvalidPlaceholder)
                    .finish()
            }
        }
    }
}

#[test]
fn test_basic() {
    use std::thread;
    let val = SemiSticky::new(true);
    assert_eq!(val.to_string(), "true");
    assert_eq!(val.get(), &true);
    assert!(val.try_get().is_ok());
    thread::spawn(move || {
        assert!(val.try_get().is_err());
    }).join()
        .unwrap();
}

#[test]
fn test_mut() {
    let mut val = SemiSticky::new(true);
    *val.get_mut() = false;
    assert_eq!(val.to_string(), "false");
    assert_eq!(val.get(), &false);
}

#[test]
fn test_drop_same_thread() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let was_called = Arc::new(AtomicBool::new(false));
    struct X(Arc<AtomicBool>);
    impl Drop for X {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let val = SemiSticky::new(X(was_called.clone()));
    drop(val);
    assert_eq!(was_called.load(Ordering::SeqCst), true);
}

#[test]
fn test_noop_drop_elsewhere() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    let was_called = Arc::new(AtomicBool::new(false));

    {
        let was_called = was_called.clone();
        thread::spawn(move || {
            struct X(Arc<AtomicBool>);
            impl Drop for X {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }

            let val = SemiSticky::new(X(was_called.clone()));
            assert!(
                thread::spawn(move || {
                    // moves it here but do not deallocate
                    val.try_get().ok();
                }).join()
                    .is_ok()
            );

            assert_eq!(was_called.load(Ordering::SeqCst), false);
        }).join()
            .unwrap();
    }

    assert_eq!(was_called.load(Ordering::SeqCst), true);
}

#[test]
fn test_rc_sending() {
    use std::rc::Rc;
    use std::thread;
    let val = SemiSticky::new(Rc::new(true));
    thread::spawn(move || {
        assert!(val.try_get().is_err());
    }).join().unwrap();
}

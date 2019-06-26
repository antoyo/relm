#![allow(deprecated)]

use std::cell::UnsafeCell;
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use super::errors::InvalidThreadAccess;

fn next_item_id() -> usize {
    static mut COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
    unsafe { COUNTER.fetch_add(1, Ordering::SeqCst) }
}

struct Registry(HashMap<usize, (UnsafeCell<*mut ()>, Box<dyn Fn(&UnsafeCell<*mut ()>)>)>);

impl Drop for Registry {
    fn drop(&mut self) {
        for (_, value) in self.0.iter() {
            (value.1)(&value.0);
        }
    }
}

thread_local!(static REGISTRY: UnsafeCell<Registry> = UnsafeCell::new(Registry(Default::default())));

/// A `Sticky<T>` keeps a value T stored in a thread.
///
/// This type works similar in nature to `Fragile<T>` and exposes the
/// same interface.  The difference is that whereas `Fragile<T>` has
/// its destructor called in the thread where the value was sent, a
/// `Sticky<T>` that is moved to another thread will have the internal
/// destructor called when the originating thread tears down.
///
/// As this uses TLS internally the general rules about the platform limitations
/// of destructors for TLS apply.
pub struct Sticky<T> {
    item_id: usize,
    _marker: PhantomData<*mut T>,
}

impl<T> Drop for Sticky<T> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            unsafe {
                if self.is_valid() {
                    self.unsafe_take_value();
                }
            }
        }
    }
}

impl<T> Sticky<T> {
    /// Creates a new `Sticky` wrapping a `value`.
    ///
    /// The value that is moved into the `Sticky` can be non `Send` and
    /// will be anchored to the thread that created the object.  If the
    /// sticky wrapper type ends up being send from thread to thread
    /// only the original thread can interact with the value.
    pub fn new(value: T) -> Self {
        let item_id = next_item_id();
        REGISTRY.with(|registry| unsafe {
            (*registry.get()).0.insert(
                item_id,
                (
                    UnsafeCell::new(Box::into_raw(Box::new(value)) as *mut _),
                    Box::new(|cell| {
                        let b: Box<T> = Box::from_raw(*(cell.get() as *mut *mut T));
                        drop(b);
                    }),
                ),
            );
        });
        Sticky {
            item_id: item_id,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn with_value<F: FnOnce(&UnsafeCell<Box<T>>) -> R, R>(&self, f: F) -> R {
        REGISTRY.with(|registry| unsafe {
            let reg = &(*(*registry).get()).0;
            if let Some(item) = reg.get(&self.item_id) {
                f(mem::transmute(&item.0))
            } else {
                panic!("trying to access wrapped value in sticky container from incorrect thread.");
            }
        })
    }

    /// Returns `true` if the access is valid.
    ///
    /// This will be `false` if the value was sent to another thread.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        // We use `try-with` here to avoid crashing if the TLS is already tearing down.
        unsafe { REGISTRY.try_with(|registry| (*registry.get()).0.contains_key(&self.item_id)).unwrap_or(false) }
    }

    #[inline(always)]
    fn assert_thread(&self) {
        if !self.is_valid() {
            panic!("trying to access wrapped value in sticky container from incorrect thread.");
        }
    }

    /// Consumes the `Sticky`, returning the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if called from a different thread than the one where the
    /// original value was created.
    pub fn into_inner(mut self) -> T {
        self.assert_thread();
        unsafe {
            let rv = self.unsafe_take_value();
            mem::forget(self);
            rv
        }
    }

    unsafe fn unsafe_take_value(&mut self) -> T {
        let ptr = REGISTRY
            .with(|registry| (*registry.get()).0.remove(&self.item_id))
            .unwrap()
            .0
            .into_inner();
        let rv = Box::from_raw(ptr as *mut T);
        *rv
    }

    /// Consumes the `Sticky`, returning the wrapped value if successful.
    ///
    /// The wrapped value is returned if this is called from the same thread
    /// as the one where the original value was created, otherwise the
    /// `Sticky` is returned as `Err(self)`.
    pub fn try_into_inner(self) -> Result<T, Self> {
        if self.is_valid() {
            Ok(self.into_inner())
        } else {
            Err(self)
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get`](#method.try_get`).
    pub fn get(&self) -> &T {
        self.with_value(|value| unsafe { &*value.get() })
    }

    /// Mutably borrows the wrapped value.
    ///
    /// # Panics
    ///
    /// Panics if the calling thread is not the one that wrapped the value.
    /// For a non-panicking variant, use [`try_get_mut`](#method.try_get_mut`).
    pub fn get_mut(&mut self) -> &mut T {
        self.with_value(|value| unsafe { &mut *value.get() })
    }

    /// Tries to immutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get(&self) -> Result<&T, InvalidThreadAccess> {
        if self.is_valid() {
            unsafe { Ok(self.with_value(|value| &*value.get())) }
        } else {
            Err(InvalidThreadAccess)
        }
    }

    /// Tries to mutably borrow the wrapped value.
    ///
    /// Returns `None` if the calling thread is not the one that wrapped the value.
    pub fn try_get_mut(&mut self) -> Result<&mut T, InvalidThreadAccess> {
        if self.is_valid() {
            unsafe { Ok(self.with_value(|value| &mut *value.get())) }
        } else {
            Err(InvalidThreadAccess)
        }
    }
}

impl<T> From<T> for Sticky<T> {
    #[inline]
    fn from(t: T) -> Sticky<T> {
        Sticky::new(t)
    }
}

impl<T: Clone> Clone for Sticky<T> {
    #[inline]
    fn clone(&self) -> Sticky<T> {
        Sticky::new(self.get().clone())
    }
}

impl<T: Default> Default for Sticky<T> {
    #[inline]
    fn default() -> Sticky<T> {
        Sticky::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for Sticky<T> {
    #[inline]
    fn eq(&self, other: &Sticky<T>) -> bool {
        *self.get() == *other.get()
    }
}

impl<T: Eq> Eq for Sticky<T> {}

impl<T: PartialOrd> PartialOrd for Sticky<T> {
    #[inline]
    fn partial_cmp(&self, other: &Sticky<T>) -> Option<cmp::Ordering> {
        self.get().partial_cmp(&*other.get())
    }

    #[inline]
    fn lt(&self, other: &Sticky<T>) -> bool {
        *self.get() < *other.get()
    }

    #[inline]
    fn le(&self, other: &Sticky<T>) -> bool {
        *self.get() <= *other.get()
    }

    #[inline]
    fn gt(&self, other: &Sticky<T>) -> bool {
        *self.get() > *other.get()
    }

    #[inline]
    fn ge(&self, other: &Sticky<T>) -> bool {
        *self.get() >= *other.get()
    }
}

impl<T: Ord> Ord for Sticky<T> {
    #[inline]
    fn cmp(&self, other: &Sticky<T>) -> cmp::Ordering {
        self.get().cmp(&*other.get())
    }
}

impl<T: fmt::Display> fmt::Display for Sticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self.get(), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Sticky<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.try_get() {
            Ok(value) => f.debug_struct("Sticky").field("value", value).finish(),
            Err(..) => {
                struct InvalidPlaceholder;
                impl fmt::Debug for InvalidPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<invalid thread>")
                    }
                }

                f.debug_struct("Sticky")
                    .field("value", &InvalidPlaceholder)
                    .finish()
            }
        }
    }
}

// similar as for fragile ths type is sync because it only accesses TLS data
// which is thread local.  There is nothing that needs to be synchronized.
unsafe impl<T> Sync for Sticky<T> {}

// The entire point of this type is to be Send
unsafe impl<T> Send for Sticky<T> {}

#[test]
fn test_basic() {
    use std::thread;
    let val = Sticky::new(true);
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
    let mut val = Sticky::new(true);
    *val.get_mut() = false;
    assert_eq!(val.to_string(), "false");
    assert_eq!(val.get(), &false);
}

#[test]
#[should_panic]
fn test_access_other_thread() {
    use std::thread;
    let val = Sticky::new(true);
    thread::spawn(move || {
        val.get();
    }).join()
        .unwrap();
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
    let val = Sticky::new(X(was_called.clone()));
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

            let val = Sticky::new(X(was_called.clone()));
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
    let val = Sticky::new(Rc::new(true));
    thread::spawn(move || {
        assert!(val.try_get().is_err());
    }).join().unwrap();
}

//! This library provides wrapper types that permit sending non `Send` types to
//! other threads and use runtime checks to ensure safety.
//!
//! It provides three types: `Fragile<T>` and `Sticky<T>` which are similar in nature
//! but have different behaviors with regards to how destructors are executed and
//! the extra `SemiSticky<T>` type which uses `Sticky<T>` if the value has a
//! destructor and `Fragile<T>` if it does not.
//!
//! Both types wrap a value and provide a `Send` bound.  Neither of the types permit
//! access to the enclosed value unless the thread that wrapped the value is attempting
//! to access it.  The difference between the two types starts playing a role once
//! destructors are involved.
//!
//! A `Fragile<T>` will actually send the `T` from thread to thread but will only
//! permit the original thread to invoke the destructor.  If the value gets dropped
//! in a different thread, the destructor will panic.
//!
//! A `Sticky<T>` on the other hand does not actually send the `T` around but keeps
//! it stored in the original thread's thread local storage.  If it gets dropped
//! in the originating thread it gets cleaned up immediately, otherwise it leaks
//! until the thread shuts down naturally.
//!
//! # Example usage
//!
//! ```
//! use std::thread;
//! use fragile::Fragile;
//!
//! // creating and using a fragile object in the same thread works
//! let val = Fragile::new(true);
//! assert_eq!(*val.get(), true);
//! assert!(val.try_get().is_ok());
//!
//! // once send to another thread it stops working
//! thread::spawn(move || {
//!     assert!(val.try_get().is_err());
//! }).join()
//!     .unwrap();
//! ```
//!
//! # Why?
//!
//! Most of the time trying to use this crate is going to indicate some code smell.  But
//! there are situations where this is useful.  For instance you might have a bunch of
//! non `Send` types but want to work with a `Send` error type.  In that case the non
//! sendable extra information can be contained within the error and in cases where the
//! error did not cross a thread boundary yet extra information can be obtained.
mod errors;
mod fragile;
mod semisticky;
mod sticky;

pub use self::errors::InvalidThreadAccess;
pub use self::fragile::Fragile;
pub use self::semisticky::SemiSticky;
pub use self::sticky::Sticky;

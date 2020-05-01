/*
 * Copyright (c) 2018 Boucher, Antoni <bouanto@zoho.com>
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

use std::mem;
use std::os::raw::c_int;
use std::ptr;

use glib::Source;
use glib::translate::{ToGlibPtr, from_glib_none};
use glib_sys::{GSource, GSourceFunc, GSourceFuncs, g_source_new};
use libc;

pub trait SourceFuncs {
    fn check(&self) -> bool {
        false
    }

    fn dispatch(&self) -> bool;
    fn prepare(&self) -> (bool, Option<u32>);
}

struct SourceData<T> {
    _source: GSource,
    funcs: Box<GSourceFuncs>,
    data: T,
}

pub fn new_source<T: SourceFuncs>(data: T) -> Source {
    unsafe {
        let mut funcs: GSourceFuncs = mem::zeroed();
        funcs.prepare = Some(prepare::<T>);
        funcs.check = Some(check::<T>);
        funcs.dispatch = Some(dispatch::<T>);
        funcs.finalize = Some(finalize::<T>);
        let mut funcs = Box::new(funcs);
        let source = g_source_new(&mut *funcs, mem::size_of::<SourceData<T>>() as u32);
        ptr::write(&mut (*(source as *mut SourceData<T>)).data, data);
        ptr::write(&mut (*(source as *mut SourceData<T>)).funcs, funcs);
        from_glib_none(source)
    }
}

pub fn source_get<T: SourceFuncs>(source: &Source) -> &T {
    unsafe { &( *(source.to_glib_none().0 as *const SourceData<T>) ).data }
}

unsafe extern "C" fn check<T: SourceFuncs>(source: *mut GSource) -> c_int {
    let object = source as *mut SourceData<T>;
    bool_to_int((*object).data.check())
}

unsafe extern "C" fn dispatch<T: SourceFuncs>(source: *mut GSource, _callback: GSourceFunc, _user_data: *mut libc::c_void)
    -> c_int
{
    let object = source as *mut SourceData<T>;
    bool_to_int((*object).data.dispatch())
}

unsafe extern "C" fn finalize<T: SourceFuncs>(source: *mut GSource) {
    println!("finalize");
    // TODO: needs a bomb to abort on panic
    let source = source as *mut SourceData<T>;
    ptr::read(&(*source).funcs);
    ptr::read(&(*source).data);
}

extern "C" fn prepare<T: SourceFuncs>(source: *mut GSource, timeout: *mut c_int) -> c_int {
    let object = source as *mut SourceData<T>;
    let (result, source_timeout) = unsafe { (*object).data.prepare() };
    if let Some(source_timeout) = source_timeout {
        unsafe { *timeout = source_timeout as i32; }
    }
    bool_to_int(result)
}

fn bool_to_int(boolean: bool) -> c_int {
    if boolean {
        1
    }
    else {
        0
    }
}

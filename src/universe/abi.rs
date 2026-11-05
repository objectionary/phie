// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

//! C ABI through which compiled Rust atoms reach the [`Universe`].
//!
//! A generated atom crate is a standalone Cargo project with no dependency
//! on `phie`, so it cannot call [`Universe`] methods directly. Instead the
//! host passes a [`UniverseVTable`]: a `#[repr(C)]` struct of `extern "C"`
//! callbacks plus an opaque context pointer. Every callback maps a graph
//! operation onto the [`Universe`] behind that pointer and reports the
//! outcome as a status code.
//!
//! The same layout is duplicated in the preamble that
//! `crate::rust_atom::compilation` writes into every generated atom crate,
//! which is what lets atom code call `uni.dataize("v1.𝛼0")` without
//! linking against `phie`.
//!
//! # Examples
//!
//! ```
//! use phie::universe::{Universe, UniverseVTable, ABI_OK};
//!
//! let mut uni = Universe::new();
//! let v = uni.next_id();
//! uni.add(v).unwrap();
//! uni.put(v, 42).unwrap();
//! let vtable = UniverseVTable::for_universe(&mut uni);
//! let found = unsafe {
//!     let path = std::ffi::CString::new(format!("v{v}")).unwrap();
//!     let mut out = 0i16;
//!     let status = (vtable.dataize)(vtable.ctx, path.as_ptr(), &mut out);
//!     (status, out)
//! };
//! assert_eq!((ABI_OK, 42), found);
//! ```

use crate::universe::Universe;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

/// Status code returned by every callback when the operation succeeded.
pub const ABI_OK: i32 = 0;

/// Status code returned by every callback when the operation failed.
pub const ABI_ERR: i32 = 1;

/// Table of `extern "C"` callbacks giving an atom access to the graph.
///
/// The struct is passed to the atom entry point as a pointer; the atom
/// invokes the fields it needs through the opaque `ctx` pointer, which
/// the host guarantees to keep valid for the whole duration of the call.
#[repr(C)]
pub struct UniverseVTable {
    /// Opaque pointer to the [`Universe`] the callbacks operate on.
    pub ctx: *mut c_void,
    /// Resolves a path such as `v1.𝛼0` and writes the data into `out`.
    pub dataize: unsafe extern "C" fn(ctx: *mut c_void, path: *const c_char, out: *mut i16) -> i32,
    /// Adds a vertex with the given ID.
    pub add: unsafe extern "C" fn(ctx: *mut c_void, v: u32) -> i32,
    /// Stores data in the given vertex.
    pub put: unsafe extern "C" fn(ctx: *mut c_void, v: u32, data: i16) -> i32,
    /// Binds the vertex `from` to the vertex `to` with a label.
    pub bind:
        unsafe extern "C" fn(ctx: *mut c_void, from: u32, to: u32, label: *const c_char) -> i32,
    /// Returns the next unique vertex ID.
    pub next_id: unsafe extern "C" fn(ctx: *mut c_void) -> u32,
}

impl UniverseVTable {
    /// Builds the callback table pointing at the given universe.
    ///
    /// # Arguments
    ///
    /// * `uni` - The universe the atom will be able to reach; it must
    ///   outlive the returned table.
    ///
    /// # Examples
    ///
    /// ```
    /// use phie::universe::{Universe, UniverseVTable};
    ///
    /// let mut uni = Universe::new();
    /// let vtable = UniverseVTable::for_universe(&mut uni);
    /// let id = unsafe { (vtable.next_id)(vtable.ctx) };
    /// assert_eq!(0, id);
    /// ```
    pub fn for_universe(uni: &mut Universe) -> Self {
        Self {
            ctx: uni as *mut Universe as *mut c_void,
            dataize: host_dataize,
            add: host_add,
            put: host_put,
            bind: host_bind,
            next_id: host_next_id,
        }
    }
}

/// Reborrows the universe behind the opaque context pointer.
///
/// # Safety
///
/// `ctx` must come from [`UniverseVTable::for_universe`] and the universe
/// must still be alive and not borrowed elsewhere.
unsafe fn universe<'a>(ctx: *mut c_void) -> &'a mut Universe {
    unsafe { &mut *(ctx as *mut Universe) }
}

/// Reads a C string into a Rust `&str`, `None` when it is not valid UTF-8.
///
/// # Safety
///
/// `text` must be a valid, NUL-terminated C string.
unsafe fn text<'a>(text: *const c_char) -> Option<&'a str> {
    if text.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(text) }.to_str().ok()
}

unsafe extern "C" fn host_dataize(ctx: *mut c_void, path: *const c_char, out: *mut i16) -> i32 {
    if out.is_null() {
        return ABI_ERR;
    }
    let Some(path) = (unsafe { text(path) }) else {
        return ABI_ERR;
    };
    match unsafe { universe(ctx) }.dataize(path) {
        Ok(data) => {
            unsafe { *out = data };
            ABI_OK
        }
        Err(_) => ABI_ERR,
    }
}

unsafe extern "C" fn host_add(ctx: *mut c_void, v: u32) -> i32 {
    match unsafe { universe(ctx) }.add(v) {
        Ok(()) => ABI_OK,
        Err(_) => ABI_ERR,
    }
}

unsafe extern "C" fn host_put(ctx: *mut c_void, v: u32, data: i16) -> i32 {
    match unsafe { universe(ctx) }.put(v, data) {
        Ok(()) => ABI_OK,
        Err(_) => ABI_ERR,
    }
}

unsafe extern "C" fn host_bind(ctx: *mut c_void, from: u32, to: u32, label: *const c_char) -> i32 {
    let Some(label) = (unsafe { text(label) }) else {
        return ABI_ERR;
    };
    match unsafe { universe(ctx) }.bind(from, to, label) {
        Ok(()) => ABI_OK,
        Err(_) => ABI_ERR,
    }
}

unsafe extern "C" fn host_next_id(ctx: *mut c_void) -> u32 {
    unsafe { universe(ctx) }.next_id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Data;
    use std::ffi::CString;

    #[test]
    fn dataize_reads_through_vtable() {
        let mut uni = Universe::new();
        let v = uni.next_id();
        uni.add(v).unwrap();
        uni.put(v, 42).unwrap();
        let vtable = UniverseVTable::for_universe(&mut uni);
        let path = CString::new(format!("v{v}")).unwrap();
        let mut out: Data = 0;
        let status = unsafe { (vtable.dataize)(vtable.ctx, path.as_ptr(), &mut out) };
        assert_eq!(ABI_OK, status);
        assert_eq!(42, out);
    }

    #[test]
    fn dataize_reports_missing_vertex() {
        let mut uni = Universe::new();
        let vtable = UniverseVTable::for_universe(&mut uni);
        let path = CString::new("v99").unwrap();
        let mut out: Data = 7;
        let status = unsafe { (vtable.dataize)(vtable.ctx, path.as_ptr(), &mut out) };
        assert_eq!(ABI_ERR, status);
        assert_eq!(7, out, "the output must stay untouched on failure");
    }

    #[test]
    fn dataize_rejects_null_arguments() {
        let mut uni = Universe::new();
        let vtable = UniverseVTable::for_universe(&mut uni);
        let mut out: Data = 0;
        let null_path = unsafe { (vtable.dataize)(vtable.ctx, std::ptr::null(), &mut out) };
        assert_eq!(ABI_ERR, null_path);
        let path = CString::new("v0").unwrap();
        let null_out = unsafe { (vtable.dataize)(vtable.ctx, path.as_ptr(), std::ptr::null_mut()) };
        assert_eq!(ABI_ERR, null_out);
    }

    #[test]
    fn add_put_bind_and_next_id_go_through_vtable() {
        let mut uni = Universe::new();
        let vtable = UniverseVTable::for_universe(&mut uni);
        let first = unsafe { (vtable.next_id)(vtable.ctx) };
        let second = unsafe { (vtable.next_id)(vtable.ctx) };
        assert_eq!(ABI_OK, unsafe { (vtable.add)(vtable.ctx, first) });
        assert_eq!(ABI_OK, unsafe { (vtable.add)(vtable.ctx, second) });
        assert_eq!(ABI_OK, unsafe { (vtable.put)(vtable.ctx, second, -3) });
        let label = CString::new("𝛼0").unwrap();
        assert_eq!(ABI_OK, unsafe {
            (vtable.bind)(vtable.ctx, first, second, label.as_ptr())
        });
        let path = CString::new(format!("v{first}.𝛼0")).unwrap();
        let mut out: Data = 0;
        let status = unsafe { (vtable.dataize)(vtable.ctx, path.as_ptr(), &mut out) };
        assert_eq!(ABI_OK, status);
        assert_eq!(-3, out);
    }

    #[test]
    fn bind_rejects_unparsable_label() {
        let mut uni = Universe::new();
        let vtable = UniverseVTable::for_universe(&mut uni);
        assert_eq!(ABI_ERR, unsafe {
            (vtable.bind)(vtable.ctx, 0, 1, std::ptr::null())
        });
    }
}

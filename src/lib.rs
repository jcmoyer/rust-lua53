// The MIT License (MIT)
//
// Copyright (c) 2014 J.C. Moyer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#![crate_name = "lua"]
#![crate_type = "lib"]

#![feature(libc, convert)]

extern crate libc;
#[macro_use]
extern crate bitflags;

pub use wrapper::state::{
  State,

  Number,
  Integer,
  Function,
  Continuation,
  Reader,
  Writer,
  Context,
  Allocator,
  Hook,
  Index,

  Arithmetic,
  Comparison,
  ThreadStatus,
  GcOption,
  Type,

  Reference,
  REFNIL, NOREF,

  HookMask,
  MASKCALL, MASKRET, MASKLINE, MASKCOUNT,

  MULTRET, REGISTRYINDEX,
  RIDX_MAINTHREAD, RIDX_GLOBALS
};

#[cfg(target_os="windows")]
mod windows {
  #[link(name="lua53")]
  extern {}
}

pub mod ffi;
pub mod wrapper;

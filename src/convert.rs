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

//! Implements conversions for Rust types to and from Lua.

use super::{State, Integer, Number, Function};

/// Trait for types that can be pushed onto the stack of a Lua state.
///
/// It is important that implementors of this trait ensure that `to_lua`
/// behaves like one of the `lua_push*` functions for consistency.
pub trait ToLua {
  /// Pushes a value of type `Self` onto the stack of a Lua state.
  fn to_lua(&self, state: &mut State);
}

impl<'a> ToLua for &'a str {
  fn to_lua(&self, state: &mut State) {
    state.push_string(*self);
  }
}

impl ToLua for String {
  fn to_lua(&self, state: &mut State) {
    state.push_string(&self);
  }
}

impl ToLua for Integer {
  fn to_lua(&self, state: &mut State) {
    state.push_integer(*self)
  }
}

impl ToLua for Number {
  fn to_lua(&self, state: &mut State) {
    state.push_number(*self)
  }
}

impl ToLua for bool {
  fn to_lua(&self, state: &mut State) {
    state.push_bool(*self)
  }
}

impl ToLua for Function {
  fn to_lua(&self, state: &mut State) {
    state.push_fn(*self)
  }
}

//#[unstable(reason="this is an experimental trait")]
impl<T> ToLua for *mut T {
  fn to_lua(&self, state: &mut State) {
    unsafe { state.push_light_userdata(*self) }
  }
}

//#[unstable(reason="this is an experimental trait")]
impl<T: ToLua> ToLua for Option<T> {
  fn to_lua(&self, state: &mut State) {
    match *self {
      Some(ref value) => value.to_lua(state),
      None            => state.push_nil(),
    }
  }
}

/// Trait for types that can be taken from the Lua stack.
///
/// It is important that implementors of this trait ensure that `from_lua`
/// behaves like one of the `lua_to*` functions for consistency.
pub trait FromLua {
  /// Converts the value on top of the stack of a Lua state to a value of type
  /// `Option<Self>`.
  fn from_lua(state: &mut State) -> Option<Self>;
}

impl FromLua for String {
  fn from_lua(state: &mut State) -> Option<String> {
    state.to_str(-1)
  }
}

impl FromLua for Integer {
  fn from_lua(state: &mut State) -> Option<Integer> {
    if state.is_integer(-1) {
      Some(state.to_integer(-1))
    } else {
      None
    }
  }
}

impl FromLua for Number {
  fn from_lua(state: &mut State) -> Option<Number> {
    if state.is_number(-1) {
      Some(state.to_number(-1))
    } else {
      None
    }
  }
}

impl FromLua for bool {
  fn from_lua(state: &mut State) -> Option<bool> {
    if state.is_bool(-1) {
      Some(state.to_bool(-1))
    } else {
      None
    }
  }
}

//#[unstable(reason="this is an experimental trait")]
impl FromLua for Function {
  fn from_lua(state: &mut State) -> Option<Function> {
    if state.is_native_fn(-1) {
      Some(state.to_native_fn(-1))
    } else {
      None
    }
  }
}


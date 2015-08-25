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

//! Implements an idiomatic, Rust wrapper around `lua_State`.
//!
//! Function names adhere to Rust naming conventions. Most of the time, this
//! means breaking up long C function names using underscores; however, there
//! are some cases where different names are used. Typically, these are cases
//! where the name itself is a reserved Rust keyword (such as `ref` in
//! `luaL_ref` or `where` in `luaL_where`) or where the name is used in both
//! the base Lua library and the auxiliary Lua library (such as
//! `lua_getmetatable` and `luaL_getmetatable`). More descriptive names have
//! been chosen for these functions. Finally, any reference to C functions has
//! been replaced by the term `native functions`. `lua_iscfunction` is
//! `is_native_fn` and `lua_tocfunction` is `to_native_fn`.

use ffi;
use ffi::{lua_State, lua_Debug};

use libc::{c_int, c_void, c_char, size_t};
use std::{mem, ptr, str, slice};
use std::ffi::{CString, CStr};
use std::borrow::Cow;
use std::borrow::ToOwned;
use super::convert::{ToLua, FromLua};

use super::{
  Number,
  Integer,
  Function,
  Allocator,
  Hook,
  Index,
};

/// Arithmetic operations for `lua_arith`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arithmetic {
  Add = ffi::LUA_OPADD as isize,
  Sub = ffi::LUA_OPSUB as isize,
  Mul = ffi::LUA_OPMUL as isize,
  Mod = ffi::LUA_OPMOD as isize,
  Pow = ffi::LUA_OPPOW as isize,
  Div = ffi::LUA_OPDIV as isize,
  IDiv = ffi::LUA_OPIDIV as isize,
  BAnd = ffi::LUA_OPBAND as isize,
  BOr = ffi::LUA_OPBOR as isize,
  BXor = ffi::LUA_OPBXOR as isize,
  Shl = ffi::LUA_OPSHL as isize,
  Shr = ffi::LUA_OPSHR as isize,
  Unm = ffi::LUA_OPUNM as isize,
  BNot = ffi::LUA_OPBNOT as isize,
}

/// Comparison operations for `lua_compare`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
  Eq = ffi::LUA_OPEQ as isize,
  Lt = ffi::LUA_OPLT as isize,
  Le = ffi::LUA_OPLE as isize,
}

/// Status of a Lua state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadStatus {
  Ok = ffi::LUA_OK as isize,
  Yield = ffi::LUA_YIELD as isize,
  RuntimeError = ffi::LUA_ERRRUN as isize,
  SyntaxError = ffi::LUA_ERRSYNTAX as isize,
  MemoryError = ffi::LUA_ERRMEM as isize,
  GcError = ffi::LUA_ERRGCMM as isize,
  MessageHandlerError = ffi::LUA_ERRERR as isize,
  FileError = ffi::LUA_ERRFILE as isize,
}

impl ThreadStatus {
  fn from_c_int(i: c_int) -> Option<ThreadStatus> {
    match i {
      ffi::LUA_OK => Some(ThreadStatus::Ok),
      ffi::LUA_YIELD => Some(ThreadStatus::Yield),
      ffi::LUA_ERRRUN => Some(ThreadStatus::RuntimeError),
      ffi::LUA_ERRSYNTAX => Some(ThreadStatus::SyntaxError),
      ffi::LUA_ERRMEM => Some(ThreadStatus::MemoryError),
      ffi::LUA_ERRGCMM => Some(ThreadStatus::GcError),
      ffi::LUA_ERRERR => Some(ThreadStatus::MessageHandlerError),
      ffi::LUA_ERRFILE => Some(ThreadStatus::FileError),
      _ => None
    }
  }

  /// Returns `true` for error statuses and `false` for `Ok` and `Yield`.
  pub fn is_err(self) -> bool {
    match self {
      ThreadStatus::RuntimeError |
      ThreadStatus::SyntaxError |
      ThreadStatus::MemoryError |
      ThreadStatus::GcError |
      ThreadStatus::MessageHandlerError |
      ThreadStatus::FileError => true,
      ThreadStatus::Ok |
      ThreadStatus::Yield => false,
    }
  }
}

/// Options for the Lua garbage collector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GcOption {
  Stop = ffi::LUA_GCSTOP as isize,
  Restart = ffi::LUA_GCRESTART as isize,
  Collect = ffi::LUA_GCCOLLECT as isize,
  Count = ffi::LUA_GCCOUNT as isize,
  CountBytes = ffi::LUA_GCCOUNTB as isize,
  Step = ffi::LUA_GCSTEP as isize,
  SetPause = ffi::LUA_GCSETPAUSE as isize,
  SetStepMul = ffi::LUA_GCSETSTEPMUL as isize,
  IsRunning = ffi::LUA_GCISRUNNING as isize,
}

/// Represents all possible Lua data types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
  None = ffi::LUA_TNONE as isize,
  Nil = ffi::LUA_TNIL as isize,
  Boolean = ffi::LUA_TBOOLEAN as isize,
  LightUserdata = ffi::LUA_TLIGHTUSERDATA as isize,
  Number = ffi::LUA_TNUMBER as isize,
  String = ffi::LUA_TSTRING as isize,
  Table = ffi::LUA_TTABLE as isize,
  Function = ffi::LUA_TFUNCTION as isize,
  Userdata = ffi::LUA_TUSERDATA as isize,
  Thread = ffi::LUA_TTHREAD as isize,
}

impl Type {
  fn from_c_int(i: c_int) -> Option<Type> {
    match i {
      ffi::LUA_TNIL => Some(Type::Nil),
      ffi::LUA_TBOOLEAN => Some(Type::Boolean),
      ffi::LUA_TLIGHTUSERDATA => Some(Type::LightUserdata),
      ffi::LUA_TNUMBER => Some(Type::Number),
      ffi::LUA_TSTRING => Some(Type::String),
      ffi::LUA_TTABLE => Some(Type::Table),
      ffi::LUA_TFUNCTION => Some(Type::Function),
      ffi::LUA_TUSERDATA => Some(Type::Userdata),
      ffi::LUA_TTHREAD => Some(Type::Thread),
      _ => None
    }
  }
}

/// Type used for Lua references generated through `luaL_ref` and `luaL_unref`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reference(c_int);

/// The value for all references to nil values.
pub const REFNIL: Reference = Reference(ffi::LUA_REFNIL);

/// A value that is guaranteed to be different from anything returned from
/// `luaL_ref`.
pub const NOREF: Reference = Reference(ffi::LUA_REFNIL);

impl Reference {
  /// Returns `true` if this reference is equal to `REFNIL`.
  pub fn is_nil_ref(self) -> bool {
    self == REFNIL
  }

  /// Returns `true` if this reference is equal to `NOREF`.
  pub fn is_no_ref(self) -> bool {
    self == NOREF
  }

  /// Convenience function that returns the value of this reference.
  pub fn value(self) -> c_int {
    let Reference(value) = self;
    value
  }
}

bitflags! {
  flags HookMask: c_int {
    const MASKCALL  = ffi::LUA_MASKCALL,
    const MASKRET   = ffi::LUA_MASKRET,
    const MASKLINE  = ffi::LUA_MASKLINE,
    const MASKCOUNT = ffi::LUA_MASKCOUNT
  }
}

/// Specifies that all results from invoking a function should be pushed onto
/// the stack.
pub const MULTRET: c_int = ffi::LUA_MULTRET;

/// Pseudo-index used to access the Lua registry.
pub const REGISTRYINDEX: Index = ffi::LUA_REGISTRYINDEX;

pub const RIDX_MAINTHREAD: Integer = ffi::LUA_RIDX_MAINTHREAD;
pub const RIDX_GLOBALS: Integer = ffi::LUA_RIDX_GLOBALS;

unsafe extern fn continue_func<F>(st: *mut lua_State, status: c_int, ctx: ffi::lua_KContext) -> c_int
  where F: FnOnce(&mut State, c_int) -> c_int
{
  mem::transmute::<_, Box<F>>(ctx)(&mut State::from_ptr(st), status)
}

/// Wraps a `lua_State`.
#[allow(non_snake_case)]
pub struct State {
  L: *mut lua_State,
  owned: bool
}

impl State {
  /// Initializes a new Lua state. This function does not open any libraries
  /// by default. Calls `luaL_newstate` internally.
  pub fn new() -> State {
    let state = unsafe { ffi::luaL_newstate() };
    State { L: state, owned: true }
  }

  /// Constructs a wrapper `State` from a raw pointer. This is suitable for use
  /// inside of native functions that accept a `lua_State` to obtain a wrapper.
  #[allow(non_snake_case)]
  pub fn from_ptr(L: *mut lua_State) -> State {
    State { L: L, owned: false }
  }

  /// Returns an unsafe pointer to the wrapped `lua_State`.
  pub fn as_ptr(&self) -> *mut lua_State {
    self.L
  }

  /// Maps to `luaL_openlibs`.
  pub fn open_libs(&mut self) {
    unsafe { ffi::luaL_openlibs(self.L) }
  }

  /// Maps to `luaL_dofile`.
  pub fn do_file(&mut self, filename: &str) -> ThreadStatus {
    let c_str = CString::new(filename).unwrap();
    let result = unsafe {
      ffi::luaL_dofile(self.L, c_str.as_ptr())
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `luaL_dostring`.
  pub fn do_string(&mut self, s: &str) -> ThreadStatus {
    let c_str = CString::new(s).unwrap();
    let result = unsafe {
      ffi::luaL_dostring(self.L, c_str.as_ptr())
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Pushes the given value onto the stack.
  pub fn push<T: ToLua>(&mut self, value: T) {
    value.to_lua(self);
  }

  /// Converts the value on top of the stack to a value of type `T` and returns
  /// it.
  pub fn to_type<T: FromLua>(&mut self) -> Option<T> {
    FromLua::from_lua(self)
  }

  //===========================================================================
  // State manipulation
  //===========================================================================
  /// Maps to `lua_close`.
  pub fn close(self) {
    // lua_close will be called in the Drop impl
    if !self.owned {
      panic!("cannot explicitly close non-owned Lua state")
    }
  }

  /// Maps to `lua_newthread`.
  pub fn new_thread(&mut self) -> State {
    let state = unsafe { ffi::lua_newthread(self.L) };
    State::from_ptr(state)
  }

  /// Maps to `lua_atpanic`.
  pub fn at_panic(&mut self, panicf: Function) -> Function {
    unsafe { ffi::lua_atpanic(self.L, panicf) }
  }

  /// Maps to `lua_version`.
  pub fn version(state: Option<&mut State>) -> Number {
    let ptr = match state {
      Some(state) => state.L,
      None        => ptr::null_mut()
    };
    unsafe { *ffi::lua_version(ptr) }
  }

  //===========================================================================
  // Basic stack manipulation
  //===========================================================================
  /// Maps to `lua_absindex`.
  pub fn abs_index(&mut self, idx: Index) -> Index {
    unsafe { ffi::lua_absindex(self.L, idx) }
  }

  /// Maps to `lua_gettop`.
  pub fn get_top(&mut self) -> Index {
    unsafe { ffi::lua_gettop(self.L) }
  }

  /// Maps to `lua_settop`.
  pub fn set_top(&mut self, index: Index) {
    unsafe { ffi::lua_settop(self.L, index) }
  }

  /// Maps to `lua_pushvalue`.
  pub fn push_value(&mut self, index: Index) {
    unsafe { ffi::lua_pushvalue(self.L, index) }
  }

  /// Maps to `lua_rotate`.
  pub fn rotate(&mut self, idx: Index, n: c_int) {
    unsafe { ffi::lua_rotate(self.L, idx, n) }
  }

  /// Maps to `lua_copy`.
  pub fn copy(&mut self, from_idx: Index, to_idx: Index) {
    unsafe { ffi::lua_copy(self.L, from_idx, to_idx) }
  }

  /// Maps to `lua_checkstack`.
  pub fn check_stack(&mut self, extra: c_int) -> bool {
    let result = unsafe { ffi::lua_checkstack(self.L, extra) };
    result != 0
  }

  /// Maps to `lua_xmove`.
  pub fn xmove(&mut self, to: &mut State, n: c_int) {
    unsafe { ffi::lua_xmove(self.L, to.L, n) }
  }

  //===========================================================================
  // Access functions (stack -> C)
  //===========================================================================
  /// Maps to `lua_isnumber`.
  pub fn is_number(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isnumber(self.L, index) == 1 }
  }

  /// Maps to `lua_isstring`.
  pub fn is_string(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isstring(self.L, index) == 1 }
  }

  /// Maps to `lua_iscfunction`.
  pub fn is_native_fn(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_iscfunction(self.L, index) == 1 }
  }

  /// Maps to `lua_isinteger`.
  pub fn is_integer(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isinteger(self.L, index) == 1 }
  }

  /// Maps to `lua_isuserdata`.
  pub fn is_userdata(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isuserdata(self.L, index) == 1 }
  }

  /// Maps to `lua_type`.
  pub fn type_of(&mut self, index: Index) -> Option<Type> {
    let result = unsafe { ffi::lua_type(self.L, index) };
    Type::from_c_int(result)
  }

  /// Maps to `lua_typename`.
  pub fn typename_of(&mut self, tp: Type) -> String {
    unsafe {
      let ptr = ffi::lua_typename(self.L, tp as c_int);
      let slice = CStr::from_ptr(ptr).to_bytes();
      str::from_utf8(slice).map(|s| s.to_owned()).unwrap()
    }
  }

  /// Maps to `lua_tonumberx`.
  pub fn to_numberx(&mut self, index: Index) -> Option<Number> {
    let mut isnum: c_int = 0;
    let result = unsafe { ffi::lua_tonumberx(self.L, index, &mut isnum) };
    if isnum == 0 {
      None
    } else {
      Some(result)
    }
  }

  /// Maps to `lua_tointegerx`.
  pub fn to_integerx(&mut self, index: Index) -> Option<Integer> {
    let mut isnum: c_int = 0;
    let result = unsafe { ffi::lua_tointegerx(self.L, index, &mut isnum) };
    if isnum == 0 {
      None
    } else {
      Some(result)
    }
  }

  /// Maps to `lua_toboolean`.
  pub fn to_bool(&mut self, index: Index) -> bool {
    let result = unsafe { ffi::lua_toboolean(self.L, index) };
    result != 0
  }

  // omitted: lua_tolstring

  /// Maps to `lua_rawlen`.
  pub fn raw_len(&mut self, index: Index) -> size_t {
    unsafe { ffi::lua_rawlen(self.L, index) }
  }

  /// Maps to `lua_tocfunction`.
  pub fn to_native_fn(&mut self, index: Index) -> Function {
    let result = unsafe { ffi::lua_tocfunction(self.L, index) };
    result
  }

  /// Maps to `lua_touserdata`.
  pub fn to_userdata(&mut self, index: Index) -> *mut c_void {
    unsafe { ffi::lua_touserdata(self.L, index) }
  }

  /// Convenience function that calls `to_userdata` and performs a cast.
  //#[unstable(reason="this is an experimental function")]
  pub unsafe fn to_userdata_typed<'a, T>(&'a mut self, index: Index) -> Option<&'a mut T> {
    mem::transmute(self.to_userdata(index))
  }

  /// Maps to `lua_tothread`.
  pub fn to_thread(&mut self, index: Index) -> Option<State> {
    let state = unsafe { ffi::lua_tothread(self.L, index) };
    if state.is_null() {
      None
    } else {
      Some(State::from_ptr(state))
    }
  }

  /// Maps to `lua_topointer`.
  pub fn to_pointer(&mut self, index: Index) -> *const c_void {
    unsafe { ffi::lua_topointer(self.L, index) }
  }

  //===========================================================================
  // Comparison and arithmetic functions
  //===========================================================================
  /// Maps to `lua_arith`.
  pub fn arith(&mut self, op: Arithmetic) {
    unsafe { ffi::lua_arith(self.L, op as c_int) }
  }

  /// Maps to `lua_rawequal`.
  pub fn raw_equal(&mut self, idx1: Index, idx2: Index) -> bool {
    let result = unsafe { ffi::lua_rawequal(self.L, idx1, idx2) };
    result != 0
  }

  /// Maps to `lua_compare`.
  pub fn compare(&mut self, idx1: Index, idx2: Index, op: Comparison) -> bool {
    let result = unsafe { ffi::lua_compare(self.L, idx1, idx2, op as c_int) };
    result != 0
  }

  //===========================================================================
  // Push functions (C -> stack)
  //===========================================================================
  /// Maps to `lua_pushnil`.
  pub fn push_nil(&mut self) {
    unsafe { ffi::lua_pushnil(self.L) }
  }

  /// Maps to `lua_pushnumber`.
  pub fn push_number(&mut self, n: Number) {
    unsafe { ffi::lua_pushnumber(self.L, n) }
  }

  /// Maps to `lua_pushinteger`.
  pub fn push_integer(&mut self, i: Integer) {
    unsafe { ffi::lua_pushinteger(self.L, i) }
  }

  // omitted: lua_pushlstring

  /// Maps to `lua_pushstring`.
  pub fn push_string(&mut self, s: &str) -> CString {
    unsafe {
      let c_str = CString::new(s).unwrap();
      ffi::lua_pushstring(self.L, c_str.as_ptr());
      c_str
    }
  }

  // omitted: lua_pushvfstring
  // omitted: lua_pushfstring

  /// Maps to `lua_pushcclosure`.
  pub fn push_closure(&mut self, f: Function, n: c_int) {
    unsafe { ffi::lua_pushcclosure(self.L, f, n) }
  }

  /// Maps to `lua_pushboolean`.
  pub fn push_bool(&mut self, b: bool) {
    unsafe { ffi::lua_pushboolean(self.L, b as c_int) }
  }

  /// Maps to `lua_pushlightuserdata`. The Lua state will receive a pointer to
  /// the given value. The caller is responsible for cleaning up the data. Any
  /// code that manipulates the userdata is free to modify its contents, so
  /// memory safety is not guaranteed.
  pub unsafe fn push_light_userdata<T>(&mut self, ud: *mut T) {
    ffi::lua_pushlightuserdata(self.L, mem::transmute(ud))
  }

  /// Maps to `lua_pushthread`.
  pub fn push_thread(&mut self) -> bool {
    let result = unsafe { ffi::lua_pushthread(self.L) };
    result != 1
  }

  //===========================================================================
  // Get functions (Lua -> stack)
  //===========================================================================
  /// Maps to `lua_getglobal`.
  pub fn get_global(&mut self, name: &str) -> Type {
    let c_str = CString::new(name).unwrap();
    let ty = unsafe {
      ffi::lua_getglobal(self.L, c_str.as_ptr())
    };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_gettable`.
  pub fn get_table(&mut self, index: Index) -> Type {
    let ty = unsafe { ffi::lua_gettable(self.L, index) };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_getfield`.
  pub fn get_field(&mut self, index: Index, k: &str) -> Type {
    let c_str = CString::new(k).unwrap();
    let ty = unsafe {
      ffi::lua_getfield(self.L, index, c_str.as_ptr())
    };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_geti`.
  pub fn geti(&mut self, index: Index, i: Integer) -> Type {
    let ty = unsafe {
      ffi::lua_geti(self.L, index, i)
    };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_rawget`.
  pub fn raw_get(&mut self, index: Index) -> Type {
    let ty = unsafe { ffi::lua_rawget(self.L, index) };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_rawgeti`.
  pub fn raw_geti(&mut self, index: Index, n: Integer) -> Type {
    let ty = unsafe { ffi::lua_rawgeti(self.L, index, n) };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_rawgetp`.
  pub fn raw_getp<T>(&mut self, index: Index, p: *const T) -> Type {
    let ty = unsafe { ffi::lua_rawgetp(self.L, index, mem::transmute(p)) };
    Type::from_c_int(ty).unwrap()
  }

  /// Maps to `lua_createtable`.
  pub fn create_table(&mut self, narr: c_int, nrec: c_int) {
    unsafe { ffi::lua_createtable(self.L, narr, nrec) }
  }

  /// Maps to `lua_newuserdata`. The pointer returned is owned by the Lua state
  /// and it will be garbage collected when it is no longer in use or the state
  /// is closed. To specify custom cleanup behavior, use a `__gc` metamethod.
  pub fn new_userdata(&mut self, sz: size_t) -> *mut c_void {
    unsafe { ffi::lua_newuserdata(self.L, sz) }
  }

  /// Convenience function that uses type information to call `new_userdata`
  /// and perform a cast.
  ///
  /// # Example
  ///
  /// ```ignore
  /// *state.new_userdata_typed() = MyStruct::new(...);
  /// state.set_metatable_from_registry(-1, "MyStruct");
  /// ```
  //#[unstable(reason="this is an experimental function")]
  pub fn new_userdata_typed<T>(&mut self) -> *mut T {
    self.new_userdata(mem::size_of::<T>() as size_t) as *mut T
  }

  /// Maps to `lua_getmetatable`.
  pub fn get_metatable(&mut self, objindex: Index) -> bool {
    let result = unsafe { ffi::lua_getmetatable(self.L, objindex) };
    result != 0
  }

  /// Maps to `lua_getuservalue`.
  pub fn get_uservalue(&mut self, idx: Index) -> Type {
    let result = unsafe { ffi::lua_getuservalue(self.L, idx) };
    Type::from_c_int(result).unwrap()
  }

  //===========================================================================
  // Set functions (stack -> Lua)
  //===========================================================================
  /// Maps to `lua_setglobal`.
  pub fn set_global(&mut self, var: &str) {
    let c_str = CString::new(var).unwrap();
    unsafe { ffi::lua_setglobal(self.L, c_str.as_ptr()) }
  }

  /// Maps to `lua_settable`.
  pub fn set_table(&mut self, idx: Index) {
    unsafe { ffi::lua_settable(self.L, idx) }
  }

  /// Maps to `lua_setfield`.
  pub fn set_field(&mut self, idx: Index, k: &str) {
    let c_str = CString::new(k).unwrap();
    unsafe { ffi::lua_setfield(self.L, idx, c_str.as_ptr()) }
  }

  /// Maps to `lua_seti`.
  pub fn seti(&mut self, idx: Index, n: Integer) {
    unsafe { ffi::lua_seti(self.L, idx, n) }
  }

  /// Maps to `lua_rawset`.
  pub fn raw_set(&mut self, idx: Index) {
    unsafe { ffi::lua_rawset(self.L, idx) }
  }

  /// Maps to `lua_rawseti`.
  pub fn raw_seti(&mut self, idx: Index, n: Integer) {
    unsafe { ffi::lua_rawseti(self.L, idx, n) }
  }

  /// Maps to `lua_rawsetp`.
  pub fn raw_setp<T>(&mut self, idx: Index, p: *const T) {
    unsafe { ffi::lua_rawsetp(self.L, idx, mem::transmute(p)) }
  }

  /// Maps to `lua_setmetatable`.
  pub fn set_metatable(&mut self, objindex: Index) {
    unsafe { ffi::lua_setmetatable(self.L, objindex) };
  }

  /// Maps to `lua_setuservalue`.
  pub fn set_uservalue(&mut self, idx: Index) {
    unsafe { ffi::lua_setuservalue(self.L, idx) }
  }

  //===========================================================================
  // 'load' and 'call' functions (load and run Lua code)
  //===========================================================================
  /// Maps to `lua_callk`.
  pub fn callk<F>(&mut self, nargs: c_int, nresults: c_int, continuation: F)
    where F: FnOnce(&mut State, c_int) -> c_int
  {
    unsafe { ffi::lua_callk(self.L, nargs, nresults, mem::transmute(Box::new(continuation)), Some(continue_func::<F>)) }
  }

  /// Maps to `lua_call`.
  pub fn call(&mut self, nargs: c_int, nresults: c_int) {
    unsafe { ffi::lua_call(self.L, nargs, nresults) }
  }

  /// Maps to `lua_pcallk`.
  pub fn pcallk<F>(&mut self, nargs: c_int, nresults: c_int, msgh: c_int, continuation: F) -> ThreadStatus
    where F: FnOnce(&mut State, c_int) -> c_int
  {
    let result = unsafe {
      ffi::lua_pcallk(self.L, nargs, nresults, msgh, mem::transmute(Box::new(continuation)), Some(continue_func::<F>))
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `lua_pcall`.
  pub fn pcall(&mut self, nargs: c_int, nresults: c_int, msgh: c_int) -> ThreadStatus {
    let result = unsafe {
      ffi::lua_pcall(self.L, nargs, nresults, msgh)
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  // TODO: mode typing?
  /// Maps to `lua_load`.
  pub fn load(&mut self, mut reader: &mut FnMut(&mut State) -> &[u8], source: &str, mode: &str) -> ThreadStatus {
    unsafe extern fn read(st: *mut lua_State, ud: *mut c_void, sz: *mut size_t) -> *const c_char {
      let mut reader: &mut &mut FnMut(&mut State) -> &[u8] = mem::transmute(ud);
      let mut state = State::from_ptr(st);
      let slice = reader(&mut state);
      *sz = slice.len() as size_t;
      slice.as_ptr() as *const _
    }
    let source_c_str = CString::new(source).unwrap();
    let mode_c_str = CString::new(mode).unwrap();
    let result = unsafe {
      ffi::lua_load(self.L, Some(read), mem::transmute(&mut reader), source_c_str.as_ptr(), mode_c_str.as_ptr())
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  // returns isize because the return value is dependent on the writer - seems to
  // be usable for anything
  /// Maps to `lua_dump`.
  pub fn dump(&mut self, mut writer: &mut FnMut(&mut State, &[u8]) -> c_int, strip: bool) -> c_int {
    unsafe extern fn write(st: *mut lua_State, p: *const c_void, sz: size_t, ud: *mut c_void) -> c_int {
      let mut writer: &mut &mut FnMut(&mut State, &[u8]) -> c_int = mem::transmute(ud);
      writer(&mut State::from_ptr(st), slice::from_raw_parts(p as *const _, sz as usize))
    }
    unsafe { ffi::lua_dump(self.L, Some(write), mem::transmute(&mut writer), strip as c_int) }
  }

  //===========================================================================
  // Coroutine functions
  //===========================================================================
  /// Maps to `lua_yieldk`.
  pub fn co_yieldk<F>(&mut self, nresults: c_int, continuation: F) -> c_int
    where F: FnOnce(&mut State, c_int) -> c_int
  {
    unsafe { ffi::lua_yieldk(self.L, nresults, mem::transmute(Box::new(continuation)), Some(continue_func::<F>)) }
  }

  /// Maps to `lua_yield`. This function is not called `yield` because it is a
  /// reserved keyword.
  pub fn co_yield(&mut self, nresults: c_int) -> c_int {
    unsafe { ffi::lua_yield(self.L, nresults) }
  }

  /// Maps to `lua_resume`.
  pub fn resume(&mut self, from: Option<&mut State>, nargs: c_int) -> c_int {
    let from_ptr = match from {
      Some(state) => state.L,
      None        => ptr::null_mut()
    };
    unsafe { ffi::lua_resume(self.L, from_ptr, nargs) }
  }

  /// Maps to `lua_status`.
  pub fn status(&mut self) -> ThreadStatus {
    let result = unsafe { ffi::lua_status(self.L) };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `lua_isyieldable`.
  pub fn is_yieldable(&mut self) -> bool {
    let result = unsafe { ffi::lua_isyieldable(self.L) };
    result != 0
  }

  //===========================================================================
  // Garbage-collection function
  //===========================================================================
  // TODO: return typing?
  /// Maps to `lua_gc`.
  pub fn gc(&mut self, what: GcOption, data: c_int) -> c_int {
    unsafe { ffi::lua_gc(self.L, what as c_int, data) }
  }

  //===========================================================================
  // Miscellaneous functions
  //===========================================================================
  /// Maps to `lua_error`.
  pub fn error(&mut self) -> c_int {
    unsafe { ffi::lua_error(self.L) }
  }

  /// Maps to `lua_next`.
  pub fn next(&mut self, idx: Index) -> bool {
    let result = unsafe { ffi::lua_next(self.L, idx) };
    result != 0
  }

  /// Maps to `lua_concat`.
  pub fn concat(&mut self, n: c_int) {
    unsafe { ffi::lua_concat(self.L, n) }
  }

  /// Maps to `lua_len`.
  pub fn len(&mut self, idx: Index) {
    unsafe { ffi::lua_len(self.L, idx) }
  }

  /// Maps to `lua_stringtonumber`.
  pub fn string_to_number(&mut self, s: &str) -> size_t {
    let c_str = CString::new(s).unwrap();
    unsafe { ffi::lua_stringtonumber(self.L, c_str.as_ptr()) }
  }

  /// Maps to `lua_getallocf`.
  pub fn get_alloc_fn(&mut self) -> (Allocator, *mut c_void) {
    let mut slot = ptr::null_mut();
    (unsafe { ffi::lua_getallocf(self.L, &mut slot) }, slot)
  }

  /// Maps to `lua_setallocf`.
  pub fn set_alloc_fn(&mut self, f: Allocator, ud: *mut c_void) {
    unsafe { ffi::lua_setallocf(self.L, f, ud) }
  }

  //===========================================================================
  // Some useful macros (here implemented as functions)
  //===========================================================================

  // omitted: lua_getextraspace

  /// Maps to `lua_tonumber`.
  pub fn to_number(&mut self, index: Index) -> Number {
    unsafe { ffi::lua_tonumber(self.L, index) }
  }

  /// Maps to `lua_tointeger`.
  pub fn to_integer(&mut self, index: Index) -> Integer {
    unsafe { ffi::lua_tointeger(self.L, index) }
  }

  /// Maps to `lua_pop`.
  pub fn pop(&mut self, n: c_int) {
    unsafe { ffi::lua_pop(self.L, n) }
  }

  /// Maps to `lua_newtable`.
  pub fn new_table(&mut self) {
    unsafe { ffi::lua_newtable(self.L) }
  }

  /// Maps to `lua_register`.
  pub fn register(&mut self, n: &str, f: Function) {
    let c_str = CString::new(n).unwrap();
    unsafe { ffi::lua_register(self.L, c_str.as_ptr(), f) }
  }

  /// Maps to `lua_pushcfunction`.
  pub fn push_fn(&mut self, f: Function) {
    unsafe { ffi::lua_pushcfunction(self.L, f) }
  }

  /// Maps to `lua_isfunction`.
  pub fn is_fn(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isfunction(self.L, index) == 1 }
  }

  /// Maps to `lua_istable`.
  pub fn is_table(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_istable(self.L, index) == 1 }
  }

  /// Maps to `lua_islightuserdata`.
  pub fn is_light_userdata(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_islightuserdata(self.L, index) == 1 }
  }

  /// Maps to `lua_isnil`.
  pub fn is_nil(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isnil(self.L, index) == 1 }
  }

  /// Maps to `lua_isboolean`.
  pub fn is_bool(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isboolean(self.L, index) == 1 }
  }

  /// Maps to `lua_isthread`.
  pub fn is_thread(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isthread(self.L, index) == 1 }
  }

  /// Maps to `lua_isnone`.
  pub fn is_none(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isnone(self.L, index) == 1 }
  }

  /// Maps to `lua_isnoneornil`.
  pub fn is_none_or_nil(&mut self, index: Index) -> bool {
    unsafe { ffi::lua_isnoneornil(self.L, index) == 1 }
  }

  // omitted: lua_pushliteral

  /// Maps to `lua_pushglobaltable`.
  pub fn push_global_table(&mut self) {
    unsafe { ffi::lua_pushglobaltable(self.L) };
  }

  /// Maps to `lua_tostring`. This function is not called `to_string` because
  /// that method name is used for the `ToString` trait. This function makes a
  /// copy of the string on top of the stack and returns it as an owned `String`.
  pub fn to_str(&mut self, index: Index) -> Option<String> {
    let ptr = unsafe { ffi::lua_tostring(self.L, index) };
    if ptr.is_null() {
      None
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      str::from_utf8(slice).map(|s| s.to_owned()).ok()
    }
  }

  /// Maps to `lua_insert`.
  pub fn insert(&mut self, idx: Index) {
    unsafe { ffi::lua_insert(self.L, idx) }
  }

  /// Maps to `lua_remove`.
  pub fn remove(&mut self, idx: Index) {
    unsafe { ffi::lua_remove(self.L, idx) }
  }

  /// Maps to `lua_replace`.
  pub fn replace(&mut self, idx: Index) {
    unsafe { ffi::lua_replace(self.L, idx) }
  }

  //===========================================================================
  // Debug API
  //===========================================================================
  /// Maps to `lua_getstack`.
  pub fn get_stack(&mut self, level: c_int) -> Option<lua_Debug> {
    let mut ar: lua_Debug = unsafe { mem::uninitialized() };
    let result = unsafe { ffi::lua_getstack(self.L, level, &mut ar) };
    if result == 0 {
      None
    } else {
      Some(ar)
    }
  }

  /// Maps to `lua_getinfo`.
  pub fn get_info(&mut self, what: &str) -> Option<lua_Debug> {
    let mut ar: lua_Debug = unsafe { mem::uninitialized() };
    let c_str = CString::new(what).unwrap();
    let result = unsafe { ffi::lua_getinfo(self.L, c_str.as_ptr(), &mut ar) };
    if result == 0 {
      None
    } else {
      Some(ar)
    }
  }

  /// Maps to `lua_getlocal`.
  pub fn get_local(&mut self, ar: &lua_Debug, n: c_int) -> Option<String> {
    let ptr = unsafe { ffi::lua_getlocal(self.L, ar, n) };
    if ptr.is_null() {
      None
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      str::from_utf8(slice).map(|s| s.to_owned()).ok()
    }
  }

  /// Maps to `lua_setlocal`.
  pub fn set_local(&mut self, ar: &lua_Debug, n: c_int) -> Option<String> {
    let ptr = unsafe { ffi::lua_setlocal(self.L, ar, n) };
    if ptr.is_null() {
      None
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      str::from_utf8(slice).map(|s| s.to_owned()).ok()
    }
  }

  /// Maps to `lua_getupvalue`.
  pub fn get_upvalue(&mut self, funcindex: Index, n: c_int) -> Option<String> {
    let ptr = unsafe { ffi::lua_getupvalue(self.L, funcindex, n) };
    if ptr.is_null() {
      None
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      str::from_utf8(slice).map(|s| s.to_owned()).ok()
    }
  }

  /// Maps to `lua_setupvalue`.
  pub fn set_upvalue(&mut self, funcindex: Index, n: c_int) -> Option<String> {
    let ptr = unsafe { ffi::lua_setupvalue(self.L, funcindex, n) };
    if ptr.is_null() {
      None
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      str::from_utf8(slice).map(|s| s.to_owned()).ok()
    }
  }

  /// Maps to `lua_upvalueid`.
  pub fn upvalue_id(&mut self, funcindex: Index, n: c_int) -> *mut c_void {
    unsafe { ffi::lua_upvalueid(self.L, funcindex, n) }
  }

  /// Maps to `lua_upvaluejoin`.
  pub fn upvalue_join(&mut self, fidx1: Index, n1: c_int, fidx2: Index, n2: c_int) {
    unsafe { ffi::lua_upvaluejoin(self.L, fidx1, n1, fidx2, n2) }
  }

  /// Maps to `lua_sethook`.
  pub fn set_hook(&mut self, func: Hook, mask: HookMask, count: c_int) {
    unsafe { ffi::lua_sethook(self.L, func, mask.bits(), count) }
  }

  /// Maps to `lua_gethook`.
  pub fn get_hook(&mut self) -> Hook {
    unsafe { ffi::lua_gethook(self.L) }
  }

  /// Maps to `lua_gethookmask`.
  pub fn get_hook_mask(&mut self) -> HookMask {
    let result = unsafe { ffi::lua_gethookmask(self.L) };
    HookMask::from_bits_truncate(result)
  }

  /// Maps to `lua_gethookcount`.
  pub fn get_hook_count(&mut self) -> c_int {
    unsafe { ffi::lua_gethookcount(self.L) }
  }

  //===========================================================================
  // Auxiliary library functions
  //===========================================================================
  /// Maps to `luaL_checkversion`.
  pub fn check_version(&mut self) {
    unsafe { ffi::luaL_checkversion(self.L) }
  }

  /// Maps to `luaL_getmetafield`.
  pub fn get_metafield(&mut self, obj: Index, e: &str) -> bool {
    let c_str = CString::new(e).unwrap();
    let result = unsafe {
      ffi::luaL_getmetafield(self.L, obj, c_str.as_ptr())
    };
    result != 0
  }

  /// Maps to `luaL_callmeta`.
  pub fn call_meta(&mut self, obj: Index, e: &str) -> bool {
    let c_str = CString::new(e).unwrap();
    let result = unsafe {
      ffi::luaL_callmeta(self.L, obj, c_str.as_ptr())
    };
    result != 0
  }

  // omitted: luaL_tolstring

  /// Maps to `luaL_argerror`.
  pub fn arg_error(&mut self, arg: Index, extramsg: &str) -> c_int {
    let c_str = CString::new(extramsg).unwrap();
    unsafe { ffi::luaL_argerror(self.L, arg, c_str.as_ptr()) }
  }

  // omitted: luaL_checklstring
  // omitted: luaL_optlstring

  /// Maps to `luaL_checknumber`.
  pub fn check_number(&mut self, arg: Index) -> Number {
    unsafe { ffi::luaL_checknumber(self.L, arg) }
  }

  /// Maps to `luaL_optnumber`.
  pub fn opt_number(&mut self, arg: Index, def: Number) -> Number {
    unsafe { ffi::luaL_optnumber(self.L, arg, def) }
  }

  /// Maps to `luaL_checkinteger`.
  pub fn check_integer(&mut self, arg: Index) -> Integer {
    unsafe { ffi::luaL_checkinteger(self.L, arg) }
  }

  /// Maps to `luaL_optinteger`.
  pub fn opt_integer(&mut self, arg: Index, def: Integer) -> Integer {
    unsafe { ffi::luaL_optinteger(self.L, arg, def) }
  }

  /// Maps to `luaL_checkstack`.
  pub fn check_stack_msg(&mut self, sz: c_int, msg: &str) {
    let c_str = CString::new(msg).unwrap();
    unsafe { ffi::luaL_checkstack(self.L, sz, c_str.as_ptr()) }
  }

  /// Maps to `luaL_checktype`.
  pub fn check_type(&mut self, arg: Index, t: Type) {
    unsafe { ffi::luaL_checktype(self.L, arg, t as c_int) }
  }

  /// Maps to `luaL_checkany`.
  pub fn check_any(&mut self, arg: Index) {
    unsafe { ffi::luaL_checkany(self.L, arg) }
  }

  /// Maps to `luaL_newmetatable`.
  pub fn new_metatable(&mut self, tname: &str) -> bool {
    let c_str = CString::new(tname).unwrap();
    let result = unsafe {
      ffi::luaL_newmetatable(self.L, c_str.as_ptr())
    };
    result != 0
  }

  /// Maps to `luaL_setmetatable`.
  pub fn set_metatable_from_registry(&mut self, tname: &str) {
    let c_str = CString::new(tname).unwrap();
    unsafe { ffi::luaL_setmetatable(self.L, c_str.as_ptr()) }
  }

  /// Maps to `luaL_testudata`.
  pub fn test_userdata(&mut self, arg: Index, tname: &str) -> *mut c_void {
    let c_str = CString::new(tname).unwrap();
    unsafe { ffi::luaL_testudata(self.L, arg, c_str.as_ptr()) }
  }

  /// Convenience function that calls `test_userdata` and performs a cast.
  //#[unstable(reason="this is an experimental function")]
  pub unsafe fn test_userdata_typed<'a, T>(&'a mut self, arg: Index, tname: &str) -> Option<&'a mut T> {
    mem::transmute(self.test_userdata(arg, tname))
  }

  /// Maps to `luaL_checkudata`.
  pub fn check_userdata(&mut self, arg: Index, tname: &str) -> *mut c_void {
    let c_str = CString::new(tname).unwrap();
    unsafe { ffi::luaL_checkudata(self.L, arg, c_str.as_ptr()) }
  }

  /// Convenience function that calls `check_userdata` and performs a cast.
  //#[unstable(reason="this is an experimental function")]
  pub unsafe fn check_userdata_typed<'a, T>(&'a mut self, arg: Index, tname: &str) -> &'a mut T {
    mem::transmute(self.check_userdata(arg, tname))
  }

  /// Maps to `luaL_where`. `where` is a reserved keyword.
  pub fn location(&mut self, lvl: c_int) {
    unsafe { ffi::luaL_where(self.L, lvl) }
  }

  // omitted: luaL_error

  /// Maps to `luaL_checkoption`.
  pub fn check_option(&mut self, arg: Index, def: Option<&str>, lst: &[&str]) -> usize {
    use std::vec::Vec;
    use libc::c_char;
    let mut vec: Vec<*const c_char> = Vec::with_capacity(lst.len() + 1);
    let cstrs: Vec<CString> = lst.iter().map(|ent| CString::new(*ent).unwrap()).collect();
    for ent in cstrs.iter() {
      vec.push(ent.as_ptr());
    }
    vec.push(ptr::null());
    let result = match def {
      Some(def) => unsafe {
        let c_str = CString::new(def).unwrap();
        ffi::luaL_checkoption(self.L, arg, c_str.as_ptr(), vec.as_ptr())
      },
      None      => unsafe {
        ffi::luaL_checkoption(self.L, arg, ptr::null(), vec.as_ptr())
      }
    };
    result as usize
  }

  /// Maps to `luaL_fileresult`.
  pub fn file_result(&mut self, stat: c_int, fname: &str) -> c_int {
    let c_str = CString::new(fname).unwrap();
    unsafe { ffi::luaL_fileresult(self.L, stat, c_str.as_ptr()) }
  }

  /// Maps to `luaL_execresult`.
  pub fn exec_result(&mut self, stat: c_int) -> c_int {
    unsafe { ffi::luaL_execresult(self.L, stat) }
  }

  /// Maps to `luaL_ref`.
  pub fn reference(&mut self, t: Index) -> Reference {
    let result = unsafe { ffi::luaL_ref(self.L, t) };
    Reference(result)
  }

  /// Maps to `luaL_unref`.
  pub fn unreference(&mut self, t: Index, reference: Reference) {
    unsafe { ffi::luaL_unref(self.L, t, reference.value()) }
  }

  /// Maps to `luaL_loadfilex`.
  pub fn load_filex(&mut self, filename: &str, mode: &str) -> ThreadStatus {
    let result = unsafe {
      let filename_c_str = CString::new(filename).unwrap();
      let mode_c_str = CString::new(mode).unwrap();
      ffi::luaL_loadfilex(self.L, filename_c_str.as_ptr(), mode_c_str.as_ptr())
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `luaL_loadfile`.
  pub fn load_file(&mut self, filename: &str) -> ThreadStatus {
    let c_str = CString::new(filename).unwrap();
    let result = unsafe {
      ffi::luaL_loadfile(self.L, c_str.as_ptr())
    };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `luaL_loadbufferx`.
  pub fn load_bufferx(&mut self, buff: &str, sz: size_t, name: &str, mode: &str) -> ThreadStatus {
    let buff_c_str = CString::new(buff).unwrap();
    let name_c_str = CString::new(name).unwrap();
    let mode_c_str = CString::new(mode).unwrap();
    let result = unsafe { ffi::luaL_loadbufferx(self.L, buff_c_str.as_ptr(), sz, name_c_str.as_ptr(), mode_c_str.as_ptr()) };
    ThreadStatus::from_c_int(result).unwrap()
  }

  /// Maps to `luaL_loadstring`.
  pub fn load_string(&mut self, source: &str) -> ThreadStatus {
    let c_str = CString::new(source).unwrap();
    let result = unsafe { ffi::luaL_loadstring(self.L, c_str.as_ptr()) };
    ThreadStatus::from_c_int(result).unwrap()
  }

  // omitted: luaL_newstate (covered by State constructor)

  /// Maps to `luaL_len`.
  pub fn len_direct(&mut self, index: Index) -> Integer {
    unsafe { ffi::luaL_len(self.L, index) }
  }

  /// Maps to `luaL_gsub`.
  pub fn gsub(&mut self, s: &str, p: &str, r: &str) -> String {
    let s_c_str = CString::new(s).unwrap();
    let p_c_str = CString::new(p).unwrap();
    let r_c_str = CString::new(r).unwrap();
    let ptr = unsafe {
      ffi::luaL_gsub(self.L, s_c_str.as_ptr(), p_c_str.as_ptr(), r_c_str.as_ptr())
    };
    let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
    str::from_utf8(slice).map(|s| s.to_owned()).unwrap()
  }

  /// Maps to `luaL_setfuncs`.
  pub fn set_fns(&mut self, l: &[(&str, Function)], nup: c_int) {
    use std::vec::Vec;
    let mut reg: Vec<ffi::luaL_Reg> = Vec::with_capacity(l.len() + 1);
    let ents: Vec<(CString, Function)> = l.iter().map(|&(s, f)| (CString::new(s).unwrap(), f)).collect();
    for &(ref s, f) in ents.iter() {
      reg.push(ffi::luaL_Reg {
        name: s.as_ptr(),
        func: f
      });
    }
    reg.push(ffi::luaL_Reg {name: ptr::null(), func: None});
    unsafe { ffi::luaL_setfuncs(self.L, reg.as_ptr(), nup) }
  }

  /// Maps to `luaL_getsubtable`.
  pub fn get_subtable(&mut self, idx: Index, fname: &str) -> bool {
    let c_str = CString::new(fname).unwrap();
    let result = unsafe {
      ffi::luaL_getsubtable(self.L, idx, c_str.as_ptr())
    };
    result != 0
  }

  /// Maps to `luaL_traceback`.
  pub fn traceback(&mut self, state: &mut State, msg: &str, level: c_int) {
    let c_str = CString::new(msg).unwrap();
    unsafe { ffi::luaL_traceback(self.L, state.L, c_str.as_ptr(), level) }
  }

  /// Maps to `luaL_requiref`.
  pub fn requiref(&mut self, modname: &str, openf: Function, glb: bool) {
    let c_str = CString::new(modname).unwrap();
    unsafe { ffi::luaL_requiref(self.L, c_str.as_ptr(), openf, glb as c_int) }
  }

  /// Maps to `luaL_newlibtable`.
  pub fn new_lib_table(&mut self, l: &[(&str, Function)]) {
    self.create_table(0, l.len() as c_int)
  }

  /// Maps to `luaL_newlib`.
  pub fn new_lib(&mut self, l: &[(&str, Function)]) {
    self.check_version();
    self.new_lib_table(l);
    self.set_fns(l, 0)
  }

  /// Maps to `luaL_argcheck`.
  pub fn arg_check(&mut self, cond: bool, arg: Index, extramsg: &str) {
    let c_str = CString::new(extramsg).unwrap();
    unsafe {
      ffi::luaL_argcheck(self.L, cond as c_int, arg, c_str.as_ptr())
    }
  }

  /// Maps to `luaL_checkstring`.
  pub fn check_string(&mut self, n: Index) -> String {
    let ptr = unsafe { ffi::luaL_checkstring(self.L, n) };
    let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
    str::from_utf8(slice).map(|s| s.to_owned()).unwrap()
  }

  /// Maps to `luaL_optstring`.
  pub fn opt_string<'a>(&mut self, n: Index, default: &'a str) -> Cow<'a, str> {
    let c_str = CString::new(default).unwrap();
    let ptr = unsafe { ffi::luaL_optstring(self.L, n, c_str.as_ptr()) };
    if ptr == c_str.as_ptr() {
      Cow::Borrowed(default)
    } else {
      let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
      let string = str::from_utf8(slice).map(|s| s.to_owned()).unwrap();
      Cow::Owned(string)
    }
  }

  // omitted: luaL_checkint (use .check_integer)
  // omitted: luaL_optint (use .opt_integer)
  // omitted: luaL_checklong (use .check_integer)
  // omitted: luaL_optlong (use .opt_integer)

  /// Maps to `luaL_typename`.
  pub fn typename_at(&mut self, n: Index) -> String {
    let ptr = unsafe { ffi::luaL_typename(self.L, n) };
    let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };
    str::from_utf8(slice).map(|s| s.to_owned()).unwrap()
  }

  // luaL_dofile and luaL_dostring implemented above

  /// Maps to `luaL_getmetatable`.
  pub fn get_metatable_from_registry(&mut self, tname: &str) {
    let c_str = CString::new(tname).unwrap();
    unsafe { ffi::luaL_getmetatable(self.L, c_str.as_ptr()) }
  }

  // omitted: luaL_opt (undocumented function)

  /// Maps to `luaL_loadbuffer`.
  pub fn load_buffer(&mut self, buff: &[u8], name: &str) -> ThreadStatus {
    let name_c_str = CString::new(name).unwrap();
    let result = unsafe { ffi::luaL_loadbuffer(self.L, buff.as_ptr() as *const _, buff.len() as size_t, name_c_str.as_ptr()) };
    ThreadStatus::from_c_int(result).unwrap()
  }

  // TODO: omitted: buffer functions
}

impl Drop for State {
  fn drop(&mut self) {
    if self.owned {
      unsafe { ffi::lua_close(self.L) }
    }
  }
}

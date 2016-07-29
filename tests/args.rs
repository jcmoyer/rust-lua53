#[macro_use]
extern crate lua;
extern crate libc;

use lua::{State, Index, Number, Integer, ThreadStatus, FromLua};
use lua::ffi::lua_State;
use libc::c_int;
use std::collections::BTreeMap;

unsafe extern "C" fn sample_function(ls: *mut lua_State) -> c_int {
    let mut state = State::from_ptr(ls);
    let (_, _) = convert_arguments!(state, String, Integer)
        .map_err(|n| state.arg_error(n, "(string, integer) expected")).unwrap();
    0
}

struct Pairs(BTreeMap<String, Number>);

impl FromLua for Pairs {
    fn from_lua(state: &mut State, index: Index) -> Option<Self> {
        let mut map = BTreeMap::new();
        let index = state.abs_index(index);
        state.push_nil();
        while state.next(index) {
            if let Ok((name, value)) = convert_arguments!(state, String, Number) {
                map.insert(name, value);
            }
            state.pop(1);
        }
        Some(Pairs(map))
    }
}

#[test]
fn test_convert_arguments_inside_fn() {
    let mut state = State::new();
    state.push_fn(Some(sample_function));
    state.set_global("sample");
    assert_eq!(state.do_string("sample(\"string\", 0)"), ThreadStatus::Ok);
    assert_eq!(state.do_string("sample(\"string\")"), ThreadStatus::RuntimeError);
    assert_eq!(state.do_string("sample(0, \"string\")"), ThreadStatus::RuntimeError);
    assert_eq!(state.do_string("sample()"), ThreadStatus::RuntimeError);
}

#[test]
fn test_convert_arguments_from_lua() {
    let mut state = State::new();
    state.do_string("return {one=1, two=2, three=3}");
    let Pairs(mut values) = state.to_type(-1).unwrap();
    assert_eq!(values.remove("one"), Some(1.0));
    assert_eq!(values.remove("two"), Some(2.0));
    assert_eq!(values.remove("three"), Some(3.0));
}


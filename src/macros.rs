use std::mem;

use libc::c_int;

use State;
use ffi::{lua_State, lua_CFunction};

/// Wrap a `fn(&mut State) -> u32` as an ffi-suitable `Function`. The argument
/// must be a path, so that the specific `fn` is known at compile-time.
#[macro_export]
macro_rules! lua_func {
  ($func:path) => { $crate::macros::_wrap(|s| $crate::macros::_check_type($func)(s)) }
}

#[doc(hidden)]
#[inline(always)]
pub fn _check_type(f: fn(&mut State) -> c_int) -> fn(&mut State) -> c_int {
  f
}

#[doc(hidden)]
#[inline]
pub fn _wrap<F: Fn(&mut State) -> c_int>(_: F) -> lua_CFunction {
  unsafe extern fn wrapped<F: Fn(&mut State) -> c_int>(s: *mut lua_State) -> c_int {
    mem::transmute::<&(), &F>(&())(&mut State::from_ptr(s))
  }
  assert!(mem::size_of::<F>() == 0, "can only wrap zero-sized closures");
  Some(wrapped::<F>)
}

/// Bind local variables with unpacked values from lua's stack.
/// As example, the following example expects lua's code calls
/// `my_function` with two arguments: string and number:
/// ```rust,norun
/// unsafe extern "C" fn my_function(ls: *mut lua_State) -> c_int {
///   let mut state = State::from_ptr(ls);
///   unpack_arguments!(state, to_str => name, to_numberx => value);
///   println!("Name argument is: {}", &name);
///   println!("Value argument is: {}", &value);
///   0
/// }
/// ```
/// If any arguments has unsuitable type or not exists `arg_error`
/// will be raised.
#[macro_export]
macro_rules! unpack_arguments {
  ($state:ident, $($conv:ident => $var:ident),*) => {
    let names = [$(stringify!($conv),)*];
    let top = $state.get_top() - names.len() as Index;
    if top < 0 {
      let no_position = $state.get_top() + 1;
      let msg = format!("{} argument(s) expected", names.len());
      $state.arg_error(no_position, &msg);
    }
    let mut position: Index = 0;
    $(let $var = {
      position += 1;
      let opt = {
        $state.$conv(top + position)
          .map(|v| v.to_owned())
      };
      match opt {
        Some(v) => v,
        None => {
          let msg = format!(
            "Can't convert value {}",
            names[(position - 1) as usize]);
          $state.arg_error(position, &msg);
        },
      }
    };)*
    // Clean up the stack (arguments still untouched)
    let new_top = $state.get_top();
    $state.pop(new_top - top);
  };
}


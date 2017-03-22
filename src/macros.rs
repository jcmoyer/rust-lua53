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

/// Cleanup stack automatically (pop all items pushed inside block).
#[macro_export]
macro_rules! auto_cleanup {
    ($state:ident, $b:block) => {{
        let top = $state.get_top();
        let result = $b;
        $state.set_top(top);
        result
    }};
}

/// Convert arguments to `FromLua` trait implementors.
/// Returns `Result<(_), Index>, which can be used to inform about errors.
///
/// FFI function usage example:
/// ```rust
/// unsafe extern "C" fn sample_function(ls: *mut lua_State) -> c_int {
///     let mut state = State::from_ptr(ls);
///     // Any `FromLua` implementor available to convert
///     let (name, delta) = convert_arguments!(state, String, Integer)
///         .map_err(|n| state.arg_error(n, "I'm expecting string and integer.")).unwrap();
///     // Do something
///     0
/// }
/// ```
///
/// `FromLua` implementation example:
/// ```rust
/// struct Pairs(BTreeMap<String, Number>);
///
/// impl FromLua for Pairs {
///     fn from_lua(state: &mut State, index: Index) -> Option<Self> {
///         let mut map = BTreeMap::new();
///         let index = state.abs_index(index);
///         state.push_nil();
///         while state.next(index) {
///             if let Ok((name, value)) = convert_arguments!(state, String, Number) {
///                 map.insert(name, value);
///             }
///             // Skip, but possible to return `None` instead
///             state.pop(1);
///         }
///         Some(Pairs(map))
///     }
/// }
/// ```
#[macro_export]
macro_rules! convert_arguments {
    ($state:ident, $($from:ty),+) => {{
        let names = [$(stringify!($from),)+];
        let quantity = names.len() as $crate::Index;
        auto_cleanup!($state, {
            let mut collect = || {
                let top = $state.get_top() - quantity;
                if top < 0 {
                    return Err(quantity + top + 1);
                }
                let mut position = 0;
                let result = ($({
                    position += 1;
                    let opt = $state.to_type::<$from>(top + position);
                    match opt {
                        Some(v) => v,
                        None => {
                            return Err(position);
                        },
                    }
                },)+);
                Ok(result)
            };
            collect()
        })
    }};
}

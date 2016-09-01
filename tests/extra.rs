extern crate lua;

use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
struct ExtraData {
    value: String,
}

impl ExtraData {
    fn wrap(self) -> Arc<lua::Extra> {
        Arc::new(Box::new(Mutex::new(self)))
    }
}

fn with_extra<F, R>(state: &mut lua::State, f: F) -> R
    where F: FnOnce(&mut ExtraData) -> R {
    let arc = state.get_extra().unwrap();
    let mutex = arc.downcast_ref::<Mutex<ExtraData>>().unwrap();
    let mut extra = mutex.lock().unwrap();
    f(&mut *extra)
}

fn unwrap_extra(state: &mut lua::State) -> ExtraData {
    let arc = state.set_extra(None).unwrap();
    let dearc = Arc::try_unwrap(arc).unwrap();
    let mutex = dearc.downcast::<Mutex<ExtraData>>().unwrap();
    let extra = mutex.into_inner().unwrap();
    extra
}

#[test]
fn test_extra_owned() {
    let mut state = lua::State::new();

    assert!(state.get_extra().is_none());
    assert!(state.set_extra(None).is_none());

    let extra = ExtraData {
      value: "Initial data".to_string(),
    };
    state.set_extra(Some(extra.wrap()));

    for x in 0..10 {
        with_extra(&mut state, |extra| {
            extra.value = format!("Changed to {}", x);
        });
    }

    assert_eq!(unwrap_extra(&mut state).value, "Changed to 9");

    assert!(state.get_extra().is_none());
    assert!(state.set_extra(None).is_none());
}

#[test]
fn test_extra_thread() {
    let mut state = lua::State::new();

    let mut thread = state.new_thread();
    assert!(thread.get_extra().is_none());
    assert!(thread.set_extra(None).is_none());
    assert!(state.get_extra().is_none());
    assert!(state.set_extra(None).is_none());

    let extra = ExtraData {
      value: "Be shared!".to_string(),
    };
    state.set_extra(Some(extra.wrap()));
    with_extra(&mut state, |extra| {
        assert_eq!(extra.value, "Be shared!");
    });

    let mut thread = state.new_thread();
    with_extra(&mut thread, |extra| {
        assert_eq!(extra.value, "Be shared!");
    });

    let local_extra = ExtraData {
      value: "I'm in thread!".to_string(),
    };
    let arc = local_extra.wrap();
    thread.set_extra(Some(arc.clone()));

    with_extra(&mut thread, |extra| {
        assert_eq!(extra.value, "I'm in thread!");
    });

    with_extra(&mut state, |extra| {
        assert_eq!(extra.value, "Be shared!");
    });

    drop(arc);
    assert!(thread.get_extra().is_none());

    with_extra(&mut state, |extra| {
        assert_eq!(extra.value, "Be shared!");
    });

}

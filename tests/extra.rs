extern crate lua;

struct ExtraData {
    value: String,
}

fn check(state: &mut lua::State) {
    let extra = ExtraData {
      value: "Initial data".to_string(),
    };

    assert!(state.get_extra().is_none());
    assert!(state.set_extra(None).is_none());

    state.set_extra(Some(Box::new(extra)));

    for x in 0..10 {
        let extra = state.get_extra()
            .and_then(|a| a.downcast_mut::<ExtraData>()).unwrap();
        extra.value = format!("Changed to {}", x);
    }

    {
        let extra = state.set_extra(None).unwrap();
        let extra = extra.downcast_ref::<ExtraData>().unwrap();
        assert_eq!(extra.value, "Changed to 9");
    }

    assert!(state.get_extra().is_none());
    assert!(state.set_extra(None).is_none());
}

#[test]
fn test_extra_owned() {
    let mut state = lua::State::new();
    check(&mut state);
}

#[test]
fn test_extra_thread() {
    let mut state = lua::State::new();
    let extra = ExtraData {
      value: "Won't be shared!".to_string(),
    };
    state.set_extra(Some(Box::new(extra)));

    let mut thread = state.new_thread();
    check(&mut thread);
}

#[test]
fn test_extra_no_impact() {
    let mut state = lua::State::new();
    let mut thread = state.new_thread();
    check(&mut thread);
    assert!(state.get_extra().is_none());
}


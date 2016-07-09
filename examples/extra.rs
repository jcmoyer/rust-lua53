extern crate lua;

struct ExtraData {
    value: String,
}

fn main() {
    let mut state = lua::State::new();
    let extra = Box::new(ExtraData {
        value: "I'm extra".to_string(),
    });

    state.open_libs();

    assert!(state.get_extra().is_none());
    assert!(state.detach_extra().is_none());

    state.attach_extra(extra);

    {
        let extra = state.get_extra()
            .and_then(|a| a.downcast_mut::<ExtraData>()).unwrap();
        println!("Value (by ref): {}", extra.value);
        extra.value = "I was changed!".to_string();
    }

    {
        let extra = state.detach_extra().unwrap();
        let extra = extra.downcast_ref::<ExtraData>().unwrap();
        println!("Value (by val): {}", extra.value);
    }

    assert!(state.get_extra().is_none());
    assert!(state.detach_extra().is_none());

}

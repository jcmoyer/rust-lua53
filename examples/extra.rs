extern crate lua;

struct Extra {
    value: String,
}

fn main() {
  let mut state = lua::State::new();
  let extra = Extra {
      value: "I'm extra".to_string(),
  };

  state.open_libs();
  state.attach_extra(extra);

  unsafe {
    let ref mut extra = *state.get_extra::<Extra>();
    println!("Value (by ref): {}", extra.value);
    extra.value = "I was changed!".to_string();

    let extra = state.detach_extra::<Extra>();
    println!("Value (by val): {}", extra.value);
  }
}

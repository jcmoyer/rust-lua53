use super::*;
use test::Bencher;

const CODE: &'static str = "\
  local array = {}
  local array2 = {}
  for x = 1, 1000 do
    table.insert(array, x)
    table.insert(array2, x)
  end
  array = nil
  arary2 = nil
";

#[bench]
fn bench_system(b: &mut Bencher) {
    let mut state = State::new();
    state.open_libs();
    b.iter(|| state.do_string(CODE));
}

#[bench]
fn bench_native(b: &mut Bencher) {
    let mut state = State::new_native();
    state.open_libs();
    b.iter(|| state.do_string(CODE));
}

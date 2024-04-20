#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/basic-pass/*.rs");
    t.compile_fail("tests/basic-fail/*.rs");
}

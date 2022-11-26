#[test]
fn run() {
    let t = trybuild::TestCases::new();
    t.pass("tests/file.rs");
    t.pass("tests/gen_rc_eq.rs");
    t.pass("tests/gen_rc_client.rs");
    //t.compile_fail("tests/gen_rc_fail.rs");
}

#[cfg(feature = "blocking")]
#[test]
fn test_gen_rc() {
    let t = trybuild::TestCases::new();
    t.pass("tests/gen_rc_eq.rs");
    t.pass("tests/gen_rc_client.rs");
}

#[test]
fn reduced_prelude_public_api() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/prelude_allowed.rs");
    cases.compile_fail("tests/ui/prelude_rejects_badge.rs");
    cases.compile_fail("tests/ui/legacy_module_hidden.rs");
}

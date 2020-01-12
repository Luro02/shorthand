use trybuild::TestCases;

#[test]
fn tests() {
    let t = TestCases::new();

    // collection tests
    t.pass("tests/collections/test_vec.rs");
    t.pass("tests/collections/test_btreemap.rs");
    t.pass("tests/collections/test_btreeset.rs");
    t.pass("tests/collections/test_hashmap.rs");
    t.pass("tests/collections/test_hashset.rs");

    //
    t.compile_fail("tests/ui/test_tuple_struct.rs");
    t.compile_fail("tests/ui/test_unit_struct.rs");
    t.compile_fail("tests/ui/test_enum.rs");
    t.compile_fail("tests/ui/test_union.rs");
    t.compile_fail("tests/ui/test_must_use.rs");
    t.compile_fail("tests/ui/test_generic_value_try_into.rs");
    t.compile_fail("tests/ui/test_generic_value_into.rs");

    // attribute errors:
    t.compile_fail("tests/ui/test_not_copy.rs");
    t.compile_fail("tests/ui/test_unknown_visibility.rs");
    t.compile_fail("tests/ui/test_redundant_disable_enable.rs");
    t.compile_fail("tests/ui/test_unknown_field.rs");
    t.compile_fail("tests/ui/test_unexpected_lit.rs");
    t.compile_fail("tests/ui/test_duplicate_enable_enable.rs");

    // rename
    t.compile_fail("tests/ui/rename/test_rename_path.rs");
    t.compile_fail("tests/ui/rename/test_rename_enable.rs");
    t.compile_fail("tests/ui/rename/test_rename_meta_name_value.rs");
    t.compile_fail("tests/ui/rename/test_rename_unexpected_lit_type.rs");
    t.compile_fail("tests/ui/rename/test_rename_pair_unexpected_lit.rs");
    t.compile_fail("tests/ui/rename/test_rename_garbage.rs");
    t.compile_fail("tests/ui/rename/test_rename_not_pair.rs");
    t.compile_fail("tests/ui/rename/test_rename_verify_format.rs");
}

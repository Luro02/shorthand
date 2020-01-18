use trybuild::TestCases;

#[test]
fn tests() {
    let t = TestCases::new();

    // collection tests
    t.pass("tests/collections/vec.rs");
    t.pass("tests/collections/btreemap.rs");
    t.pass("tests/collections/btreeset.rs");
    t.pass("tests/collections/hashmap.rs");
    t.pass("tests/collections/hashset.rs");

    // primitive_copy
    t.pass("tests/primitive_copy/array.rs");
    t.pass("tests/primitive_copy/paren.rs");
    t.pass("tests/primitive_copy/reference.rs");
    t.pass("tests/primitive_copy/tuple.rs");

    // forward
    t.compile_fail("tests/forward/parse.rs");

    //
    t.compile_fail("tests/ui/tuple_struct.rs");
    t.compile_fail("tests/ui/unit_struct.rs");
    t.compile_fail("tests/ui/enum.rs");
    t.compile_fail("tests/ui/union.rs");
    t.compile_fail("tests/ui/must_use.rs");
    t.compile_fail("tests/ui/generic_value_try_into.rs");
    t.compile_fail("tests/ui/generic_value_into.rs");
    t.compile_fail("tests/ui/shorthand.rs");

    // attribute errors:
    t.compile_fail("tests/ui/not_copy.rs");
    t.compile_fail("tests/ui/unknown_visibility.rs");
    t.compile_fail("tests/ui/redundant_disable_enable.rs");
    t.compile_fail("tests/ui/unknown_field.rs");
    t.compile_fail("tests/ui/unexpected_lit.rs");
    t.compile_fail("tests/ui/duplicate_enable_enable.rs");

    // rename
    t.compile_fail("tests/ui/rename/rename_path.rs");
    t.compile_fail("tests/ui/rename/rename_enable.rs");
    t.compile_fail("tests/ui/rename/rename_meta_name_value.rs");
    t.compile_fail("tests/ui/rename/rename_unexpected_lit_type.rs");
    t.compile_fail("tests/ui/rename/rename_pair_unexpected_lit.rs");
    t.compile_fail("tests/ui/rename/rename_garbage.rs");
    t.compile_fail("tests/ui/rename/rename_not_pair.rs");
    t.compile_fail("tests/ui/rename/rename_verify_format.rs");
}

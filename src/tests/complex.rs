use super::assert_gen;

#[test]
fn ftl_doc() {
    let ftl = r#"

key = { $var ->
    [key1] Value 1
   *[other] Value 2
}
   
    "#;
    assert_gen(module_path!(), None, true, ftl);
}

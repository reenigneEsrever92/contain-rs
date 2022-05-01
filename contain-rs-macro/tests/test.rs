
#[test]
fn test_images() {
    let t = trybuild::TestCases::new();
    t.pass("tests/images/simple_image.rs");
}
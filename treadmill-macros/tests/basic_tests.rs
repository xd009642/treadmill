#[treadmill_macros::test]
async fn simple_test() {
    // We should be in a runtime so therefore can get a handle
    let _rt = treadmill::Runtime::current();
}

#[test]
fn make_a_runtime() {
    main_function()
}

#[treadmill_macros::main]
async fn main_function() {
    // We should be in a runtime so therefore can get a handle
    let _rt = treadmill::Runtime::current();
}

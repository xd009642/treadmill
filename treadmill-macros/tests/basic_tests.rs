use std::thread;

#[treadmill_macros::test]
async fn simple_test() {
    // We should be in a runtime so therefore can get a handle
    let rt = treadmill::Runtime::current();
    assert!(!rt.is_empty());

    let res = thread::spawn(|| {
        treadmill::Runtime::try_current();
    })
    .join();

    assert!(res.is_err());
}

#[test]
fn make_a_runtime() {
    main_function()
}

#[treadmill_macros::main]
async fn main_function() {
    // We should be in a runtime so therefore can get a handle
    let rt = treadmill::Runtime::current();
    assert!(!rt.is_empty())
}

#[test]
fn no_runtime() {
    let rt = treadmill::Runtime::current();
    assert!(rt.is_empty())
}

use super::*;

#[test]
fn test_single_threaded_runtime() {
    let rt = single_threaded_runtime().expect("Failed to create single threaded runtime");
    assert_eq!(
        rt.metrics().num_workers(),
        1,
        "Expected single threaded runtime to have 1 worker"
    );
}

#[test]
fn test_multi_threaded_runtime() {
    let rt = multi_threaded_runtime().expect("Failed to create multi threaded runtime");
    assert!(
        rt.metrics().num_workers() > 1,
        "Expected multi threaded runtime to have more than 1 worker"
    );
}

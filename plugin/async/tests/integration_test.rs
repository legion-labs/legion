use std::time::Duration;

use legion_async::{AsyncOperationError, TokioAsyncRuntime};

use ntest::timeout;

#[test]
#[timeout(1000)]
fn test_async_operation() {
    let mut rt = TokioAsyncRuntime::default();

    let op = rt.start(async { 42 });

    // Even though the async method actually returns super fast, nothing polls
    // the runtime so the operation can't possibly have a result yet.
    assert!(op.take_result().is_none());

    while rt.poll() == 0 {}

    // Make sure we get the expected value.
    assert_eq!(op.take_result().unwrap_or(Ok(0)), Ok(42));

    // The second time around, the value should not be here anymore.
    assert!(op.take_result().is_none());
}

#[test]
#[timeout(1000)]
fn test_async_operation_cancellation() {
    let mut rt = TokioAsyncRuntime::default();

    let op = rt.start(async {
        // Let's give some time for the cancellation to be handled.
        tokio::time::sleep(Duration::from_secs(1)).await;
        42
    });

    op.cancel();

    // Even though the async method actually returns super fast, nothing polls
    // the runtime so the operation can't possibly have a result yet.
    assert!(op.take_result().is_none());

    while rt.poll() == 0 {}

    // Make sure we get the expected value.
    assert_eq!(
        op.take_result().unwrap_or(Ok(0)),
        Err(AsyncOperationError::Cancelled)
    );

    // The second time around, the value should not be here anymore.
    assert!(op.take_result().is_none());
}

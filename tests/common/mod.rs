use tracing::subscriber::SetGlobalDefaultError;

/// Initialize logging with the given level.
pub fn init_subscriber(level: tracing::Level) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_test_writer()
        .without_time()
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

/// Initialize the subscriber for the tests.
///
/// Cannot pass options, since the tests run concurrently.
pub fn init_tracing() {
    let level = tracing::Level::DEBUG;
    match init_subscriber(level) {
        Ok(_) => (),
        Err(_e) => (),
    }
}

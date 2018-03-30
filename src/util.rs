use std::error::Error;

/// Report error and continue.
pub fn report_error<T, E: Error>(result: Result<T, E>) -> Option<T> {
    match result {
        Ok(x) => Some(x),
        Err(e) => {
            error!("{}", e.description());
            None
        }
    }
}

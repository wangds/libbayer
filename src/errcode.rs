//! Bayer error codes.

use std::io;

pub type BayerResult<T> = Result<T, BayerError>;

quick_error! {

#[derive(Debug)]
pub enum BayerError {
    // Generic failure.  Please try to make something more meaningful.
    NoGood {
        description("No good")
    }

    Io(err: io::Error) {
        from()
        description(err.description())
        display("IO error: {}", err)
        cause(err)
    }
}

}

//! Bayer error codes.

use quick_error::quick_error;
use std::io;

pub type BayerResult<T> = Result<T, BayerError>;

quick_error! {
    #[derive(Debug)]
    pub enum BayerError {
        NoGood {
            display("No good")
        }

        WrongResolution {
            display("Wrong resolution")
        }
        WrongDepth {
            display("Wrong depth")
        }

        Io(err: io::Error) {
            from()
            display("IO error: {}", err)
        }
    }
}

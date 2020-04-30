pub use log::{debug, info, warn, error};
pub use failure::{Error, ResultExt};
pub type Result<T> = ::std::result::Result<T, Error>;

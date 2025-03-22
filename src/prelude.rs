#![allow(unused)]

pub(crate) use tracing::{debug, error, info, warn};

pub(crate) use crate::Error;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;


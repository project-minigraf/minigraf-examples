#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Schema validation for data stored in Minigraf.
//!
//! This crate lives outside `minigraf` core. Schema validation is an ecosystem
//! utility: useful for applications that want to enforce data contracts, but it
//! should not couple storage internals to application-level validation choices.

use std::collections::HashMap;

use anyhow::{Result, bail};
use minigraf::{Minigraf, QueryResult, Value};

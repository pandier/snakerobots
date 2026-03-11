mod async_runtime;
mod client;
mod game;
mod error;

use godot::prelude::*;

pub use crate::async_runtime::{AsyncRuntime, SrFuture};
pub use crate::error::{SrResult};

struct SnakerobotsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SnakerobotsExtension {}

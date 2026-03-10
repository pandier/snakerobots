mod async_runtime;
mod client;
mod game;

use godot::prelude::*;

pub use crate::async_runtime::{AsyncRuntime, SrFuture};

struct SnakerobotsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SnakerobotsExtension {}

mod async_runtime;
mod client;
mod error;
mod game;
mod util;

use godot::prelude::*;

use crate::async_runtime::AsyncDispatcher;
pub use crate::async_runtime::{AsyncRuntime, SrFuture};
pub use crate::error::SrResult;

struct SnakerobotsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SnakerobotsExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::MainLoop {
            AsyncDispatcher::ensure_created();
        }
    }
}

mod async_runtime;
mod game;

use godot::{classes::Engine, prelude::*};

pub use crate::async_runtime::AsyncRuntime;

struct SnakerobotsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SnakerobotsExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Scene {
            Engine::singleton().register_singleton(
                &AsyncRuntime::class_id().to_string_name(),
                &AsyncRuntime::new_alloc(),
            );
        }
    }

    fn on_stage_deinit(stage: InitStage) {
        if stage == InitStage::Scene {
            let mut engine = Engine::singleton();
            let name = AsyncRuntime::class_id().to_string_name();

            if let Some(singleton) = engine.get_singleton(&name) {
                engine.unregister_singleton(&name);
                singleton.free();
            }
        }
    }
}

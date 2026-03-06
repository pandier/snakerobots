use godot::{classes::Engine, prelude::*};
use tokio::sync::oneshot;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct AsyncRuntime {
    handle: tokio::runtime::Handle,
    _cancel: oneshot::Sender<()>,
}

impl AsyncRuntime {
    pub fn runtime() -> tokio::runtime::Handle {
        Engine::singleton()
            .get_singleton(&Self::class_id().to_string_name())
            .expect("missing `AsyncRuntime` singleton")
            .cast::<AsyncRuntime>()
            .bind()
            .handle
            .clone()
    }
}

#[godot_api]
impl IObject for AsyncRuntime {
    fn init(_base: Base<Object>) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        let handle = runtime.handle().clone();
        let (tx, rx) = oneshot::channel();

        std::thread::spawn(move || {
            let _ = runtime.block_on(rx);
        });

        Self {
            handle,
            _cancel: tx,
        }
    }
}

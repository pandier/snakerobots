use godot::{classes::Engine, prelude::*};
use smol::Task;

#[derive(GodotClass)]
#[class(singleton)]
pub struct AsyncRuntime {
    executor: smol::LocalExecutor<'static>,
}

#[godot_api]
impl IObject for AsyncRuntime {
    fn init(_base: Base<Object>) -> Self {
        Self {
            executor: smol::LocalExecutor::new(),
        }
    }
}

#[godot_api]
impl AsyncRuntime {
    pub fn spawn_gd<T: 'static>(future: impl Future<Output = T> + 'static) -> Gd<SrFuture>
    where
        T: ToGodot,
    {
        let gd_future = SrFuture::new_gd();
        let mut gd_future_clone = gd_future.clone();

        Self::spawn(async move {
            gd_future_clone
                .bind_mut()
                .complete(future.await.to_variant());
        })
        .detach();

        gd_future
    }

    pub fn spawn<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
        let inst = Self::singleton();
        inst.bind().executor().spawn(future)
    }

    pub fn executor(&self) -> &smol::LocalExecutor<'static> {
        &self.executor
    }
}

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AsyncDispatcher;

#[godot_api]
impl AsyncDispatcher {
    pub fn ensure_created() {
        assert!(
            godot::sys::is_main_thread(),
            "`ensure_created` must be called on the main thread"
        );

        if let Some(mut root) = Engine::singleton()
            .get_main_loop()
            .and_then(|x| x.try_cast::<SceneTree>().ok())
            .and_then(|x| x.get_root()) {

            let name = Self::class_id().to_string_name();
            if root.get_node_or_null(&NodePath::from(&name)).is_none() {
                let mut node = AsyncDispatcher::new_alloc();
                node.set_name(&name);
                root.add_child(&node);
            }
        }
    }
}

#[godot_api]
impl INode for AsyncDispatcher {
    fn process(&mut self, _delta: f64) {
        let runtime_gd = AsyncRuntime::singleton();
        let runtime = runtime_gd.bind();
        let executor = runtime.executor();
        while executor.try_tick() {}
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrFuture {
    result: Option<Variant>,
    base: Base<RefCounted>,
}

#[godot_api]
impl SrFuture {
    #[signal]
    fn completed(result: Variant);

    #[func]
    pub fn complete(&mut self, result: Variant) -> bool {
        if self.result.is_some() {
            false
        } else {
            self.result = Some(result.clone());
            self.signals().completed().emit(&result);
            true
        }
    }

    #[func]
    pub fn get_result(&self) -> Variant {
        self.result.clone().unwrap_or(Variant::nil())
    }

    #[func]
    pub fn is_completed(&self) -> bool {
        self.result.is_some()
    }
}

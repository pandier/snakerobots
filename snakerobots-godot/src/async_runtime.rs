use godot::{classes::Engine, prelude::*};
use smol::Task;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct AsyncRuntime {
    executor: smol::LocalExecutor<'static>,
}

#[godot_api]
impl AsyncRuntime {
    pub fn instance() -> Gd<Self> {
        assert!(
            godot::sys::is_main_thread(),
            "`try_instance` must be called on the main thread"
        );

        let Some(mut root) = Engine::singleton()
            .get_main_loop()
            .and_then(|x| x.try_cast::<SceneTree>().ok())
            .and_then(|x| x.get_root())
        else {
            panic!("scene tree is not available yet");
        };

        // TODO: make runtime singleton and create a separate node as dispatcher

        godot_print!("children: {:?}", root.get_children().iter_shared().map(|x| x.get_name().to_string()).collect::<Vec<_>>());

        let name = Self::class_id().to_string_name();

        if let Some(node) = root.get_node_or_null(&NodePath::from(&name)) {
            node.cast()
        } else {
            let mut node = AsyncRuntime::new_alloc();
            node.set_name(&name);

            let node_cloned = node.clone();
            root.run_deferred_gd(move |mut root| {
                root.add_child(&node_cloned);
            });

            node
        }
    }

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
        let inst = Self::instance();
        inst.bind().executor().spawn(future)
    }

    pub fn executor(&self) -> &smol::LocalExecutor<'static> {
        &self.executor
    }
}

#[godot_api]
impl INode for AsyncRuntime {
    fn process(&mut self, _delta: f64) {
        while self.executor.try_tick() {}
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

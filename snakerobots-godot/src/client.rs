use std::rc::Rc;

use godot::prelude::*;

use crate::{AsyncRuntime, SrFuture};

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct SrClient {
    client: Rc<surf::Client>,
}

#[godot_api]
impl IRefCounted for SrClient {
    fn init(_base: Base<RefCounted>) -> Self {
        let client = Rc::new(surf::client());
        Self { client }
    }
}

#[godot_api]
impl SrClient {
    #[inline]
    fn spawn_client<T, F>(&self, f: F) -> Gd<SrFuture>
    where
        T: ToGodot + 'static,
        F: AsyncFn(Rc<surf::Client>) -> T + 'static,
    {
        let client = self.client.clone();
        AsyncRuntime::spawn_gd(async move { f(client).await })
    }
}

use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrResult {
    #[var]
    pub value: Variant,
    #[var]
    pub err: Variant,
}

#[godot_api]
impl SrResult {
    #[func]
    pub fn value(value: Variant) -> Gd<Self> {
        Gd::from_object(Self {
            value,
            err: Variant::nil(),
        })
    }

    #[func]
    pub fn err(err: Variant) -> Gd<Self> {
        Gd::from_object(Self {
            value: Variant::nil(),
            err: err,
        })
    }

    pub fn from<T, E>(result: Result<T, E>) -> Gd<Self>
    where
        T: ToGodot,
        E: ToGodot,
    {
        match result {
            Ok(v) => SrResult::value(v.to_variant()),
            Err(e) => SrResult::err(e.to_variant()),
        }
    }

    pub fn run<V, F>(f: F) -> Gd<Self>
    where
        V: ToGodot,
        F: FnOnce() -> Result<V, Variant>,
    {
        Self::from(f())
    }
}

#[godot_api]
impl IRefCounted for SrResult {
    fn to_string(&self) -> GString {
        match self.err.is_nil() {
            true => format!("value({})", self.value).to_godot(),
            false => format!("err({})", self.err).to_godot(),
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrGenericError {
    #[var]
    pub message: GString,
}

#[godot_api]
impl SrGenericError {
    #[func]
    pub fn create(message: GString) -> Gd<Self> {
        Gd::from_object(Self {
            message
        })
    } 
}

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
            err,
        })
    }

    pub fn from<T, E>(result: Result<T, E>) -> Gd<Self> where T: ToGodot, E: std::fmt::Display {
        match result {
            Ok(v) => SrResult::value(v.to_variant()),
            Err(err) => SrResult::err(format!("{}", err).to_variant()),
        }
    }
}

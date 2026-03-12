use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrResult {
    #[var]
    pub value: Variant,
    #[var]
    pub err: Option<Gd<SrError>>,
}

#[godot_api]
impl SrResult {
    #[func]
    pub fn value(value: Variant) -> Gd<Self> {
        Gd::from_object(Self { value, err: None })
    }

    #[func]
    pub fn err(err: Gd<SrError>) -> Gd<Self> {
        Gd::from_object(Self {
            value: Variant::nil(),
            err: Some(err),
        })
    }

    pub fn from<T, E>(result: Result<T, E>) -> Gd<Self>
    where
        T: ToGodot,
        E: Into<SrError>,
    {
        match result {
            Ok(v) => SrResult::value(v.to_variant()),
            Err(err) => SrResult::err(Gd::from_object(err.into())),
        }
    }
}

#[godot_api]
impl IRefCounted for SrResult {
    fn to_string(&self) -> GString {
        match &self.err {
            Some(err) => format!("err({})", err).to_godot(),
            None => format!("value({})", self.value.to_string()).to_godot(),
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrError {
    #[var]
    pub code: GString,
    #[var]
    pub message: GString,
}

#[godot_api]
impl SrError {
    #[func]
    pub fn new(code: GString, message: GString) -> Gd<Self> {
        Gd::from_object(Self { code, message })
    }
}

#[godot_api]
impl IRefCounted for SrError {
    fn to_string(&self) -> GString {
        format!("{{code=\"{}\",message=\"{}\"}}", self.code, self.message).to_godot()
    }
}

impl<E: std::fmt::Display> From<E> for SrError {
    fn from(value: E) -> Self {
        Self {
            code: "unknown".to_godot(),
            message: format!("{}", value).to_godot(),
        }
    }
}

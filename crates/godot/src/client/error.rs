use godot::{prelude::*, meta::ToGodot};

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrClientError {
    #[var]
    pub code: GString,
    #[var]
    pub message: GString,
}

#[godot_api]
impl SrClientError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_godot(),
            message: message.to_godot(),
        }
    }

    pub fn unknown(message: &str) -> Self {
        Self::new("unknown", message)
    }

    pub fn unauthorized() -> Self {
        Self::new("unauthorized", "Unauthorized")
    }
}

#[godot_api]
impl IRefCounted for SrClientError {
    fn to_string(&self) -> GString {
        format!("{{code=\"{}\",message=\"{}\"}}", self.code, self.message).to_godot()
    }
}

impl<E: std::fmt::Display> From<E> for SrClientError {
    fn from(value: E) -> Self {
        Self::unknown(&value.to_string())
    }
}

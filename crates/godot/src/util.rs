use chrono::{DateTime, Local};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrUtil {
}

#[godot_api]
impl SrUtil {

    #[func]
    pub fn format_unix_timestamp(unix: i64) -> String {
        if let Some(dt) = DateTime::from_timestamp(unix, 0) {
            let nt = dt.with_timezone(&Local);
            return nt.format("%H:%M:%S, %e %b %Y").to_string();
        }
        return "<invalid>".into();
    }
}
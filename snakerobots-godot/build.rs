use std::env;

fn main() {
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_path("../deployment/dev/.env");

    expose_env("API_URL");
    expose_env("DEV_API_URL");
    expose_env("DEV_TOKEN");
}

fn expose_env(key: &str) {
    if let Ok(value) = env::var(key) {
        println!("cargo:rustc-env={}={}", key, value);
    }
}

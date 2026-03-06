use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};

pub async fn hash_password(password: String) -> eyre::Result<String> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| eyre::eyre!("{}", e))
    }).await?
} 

pub async fn verify_password(password: String, hash: String) -> eyre::Result<bool> {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map_err(|e| eyre::eyre!("Failed to parse password hash: {}", e))
            .map(|parsed_hash| {
                Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
            })
    }).await?
}

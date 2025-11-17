use anyhow::Result;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error as PwHashError, SaltString};
use argon2::{Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier};

/// Configuration for Argon2 parameters
struct ArgonConfig {
    memory_cost_kib: u32,
    time_cost: u32,
    parallelism: u32,
}

impl Default for ArgonConfig {
    fn default() -> Self {
        Self {
            memory_cost_kib: 19456,
            time_cost: 2,
            parallelism: 1,
        }
    }
}

/// Hashes a password
/// Returns the full hash string which includes the salt and parameters
pub fn hash_password(password: &str) -> Result<String, PwHashError> {
    let password_bytes = password.as_bytes();

    let salt = SaltString::generate(&mut OsRng);

    let config = ArgonConfig::default();

    let params = Params::new(
        config.memory_cost_kib,
        config.time_cost,
        config.parallelism,
        None,
    )
    .map_err(|_err| PwHashError::ParamNameInvalid)?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    Ok(argon2.hash_password(password_bytes, &salt)?.to_string())
}

/// Verifies a password against a stored Argon2 hash
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, PwHashError> {
    let password_bytes = password.as_bytes();

    let parsed_hash = PasswordHash::new(stored_hash)?;

    let verification_result = Argon2::default().verify_password(password_bytes, &parsed_hash);

    match verification_result {
        Ok(()) => Ok(true),
        Err(PwHashError::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test generates a password hash for a dummy admin account.
    /// Run it with `cargo test -- --nocapture` to see the output hash.
    #[test]
    fn generate_dummy_admin_hash() {
        let password = "user123";
        let hash_result = hash_password(password);

        // Ensure the hashing process was successful
        assert!(hash_result.is_ok());

        let hash = hash_result.unwrap();
        println!("\nGenerated hash for '{}': {}\n", password, hash);

        let is_valid = verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }
}

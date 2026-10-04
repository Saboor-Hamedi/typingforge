use crate::db::{DatabaseConnection, DbQueries, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct LocalAuth;

impl LocalAuth {
    /// Hashes a plaintext password using Argon2id with a secure random salt.
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| format!("Password hashing failed: {e}"))
    }

    /// Verifies a plaintext password against an Argon2 password hash.
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Registers a new local user in the SQLite database.
    pub fn register(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<User, String> {
        let trimmed_user = username.trim();
        if trimmed_user.chars().count() < 2 {
            return Err("Username must be at least 2 characters long".to_string());
        }
        if trimmed_user.chars().count() > 32 {
            return Err("Username must be at most 32 characters long".to_string());
        }
        if trimmed_user.chars().any(char::is_whitespace) {
            return Err("Username cannot contain spaces".to_string());
        }
        if password.chars().count() < 3 {
            return Err("Password must be at least 3 characters long".to_string());
        }

        // Check if username is already taken (unique check)
        if let Ok(Some(_)) = DbQueries::get_user_by_username(db, trimmed_user) {
            return Err("Username is already taken. Please choose another username.".to_string());
        }

        let hash = Self::hash_password(password)?;
        DbQueries::create_user(db, trimmed_user, &hash).map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("UNIQUE constraint failed") {
                "Username is already taken. Please choose another username.".to_string()
            } else {
                format!("Database error creating user: {e}")
            }
        })
    }

    /// Authenticates a local user with their username and password.
    pub fn authenticate(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<User, String> {
        let trimmed_user = username.trim();
        let user = DbQueries::get_user_by_username(db, trimmed_user)
            .map_err(|e| format!("Database error: {e}"))?
            .ok_or_else(|| "User not found".to_string())?;

        if Self::verify_password(password, &user.password_hash) {
            Ok(user)
        } else {
            Err("Incorrect password".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "super_secret_typing_key";
        let hash = LocalAuth::hash_password(password).expect("hash password");
        assert!(LocalAuth::verify_password(password, &hash));
        assert!(!LocalAuth::verify_password("wrong_password", &hash));
    }
}

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

/// Servicio para hashear y verificar contraseñas usando Argon2
pub struct PasswordService;

impl PasswordService {
    /// Hashea una contraseña usando Argon2
    ///
    /// # Argumentos
    /// * `password` - Contraseña en texto plano
    ///
    /// # Retorna
    /// String con el hash de la contraseña en formato PHC
    ///
    /// # Errores
    /// Retorna error si el hashing falla
    pub fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Error al hashear contraseña: {}", e))?;

        Ok(password_hash.to_string())
    }

    /// Verifica si una contraseña coincide con su hash
    ///
    /// # Argumentos
    /// * `password` - Contraseña en texto plano
    /// * `hash` - Hash almacenado (en formato PHC)
    ///
    /// # Retorna
    /// `true` si la contraseña es correcta, `false` si no
    ///
    /// # Errores
    /// Retorna error si el formato del hash es inválido
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| format!("Hash inválido: {}", e))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "mi_password_segura_123";
        let hash = PasswordService::hash_password(password);

        assert!(hash.is_ok());
        let hash = hash.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2")); // Formato PHC
    }

    #[test]
    fn test_verify_password_correcto() {
        let password = "test_password_123";
        let hash = PasswordService::hash_password(password).unwrap();

        let result = PasswordService::verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrecto() {
        let password = "correct_password";
        let hash = PasswordService::hash_password(password).unwrap();

        let result = PasswordService::verify_password("wrong_password", &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_hashes_diferentes_para_misma_password() {
        let password = "same_password";
        let hash1 = PasswordService::hash_password(password).unwrap();
        let hash2 = PasswordService::hash_password(password).unwrap();

        // Los hashes deben ser diferentes (por el salt aleatorio)
        assert_ne!(hash1, hash2);

        // Pero ambos deben validar correctamente
        assert!(PasswordService::verify_password(password, &hash1).unwrap());
        assert!(PasswordService::verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_verify_hash_invalido() {
        let result = PasswordService::verify_password("password", "invalid_hash");
        assert!(result.is_err());
    }
}

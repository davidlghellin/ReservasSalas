use crate::error::{convertir_errores_validacion, SalaError};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Sala {
    pub id: String,
    #[validate(length(min = 1, max = 100))]
    pub nombre: String,
    #[validate(range(min = 1, max = 1000, message = "Sobrepasa la capacidad"))]
    pub capacidad: u32,
    pub activa: bool,
}

impl Sala {
    pub fn new(id: String, nombre: String, capacidad: u32) -> Result<Self, SalaError> {
        let mut errores: Vec<String> = Vec::new();

        let nombre_trim = nombre.trim().to_string();

        // Creamos la sala igualmente; las validaciones de `validator` van después
        let sala = Self {
            id,
            nombre: nombre_trim,
            capacidad,
            activa: true,
        };

        // Validaciones de `validator`
        if let Err(e) = sala.validate() {
            errores.extend(convertir_errores_validacion(e));
        }

        // Si hay errores acumulados, devolvemos todos a la vez
        if !errores.is_empty() {
            return Err(SalaError::Validacion(errores));
        }

        Ok(sala)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn nombre(&self) -> &str {
        &self.nombre
    }

    pub fn capacidad(&self) -> u32 {
        self.capacidad
    }

    pub fn esta_activa(&self) -> bool {
        self.activa
    }

    pub fn activar(&mut self) {
        self.activa = true;
    }

    pub fn desactivar(&mut self) {
        self.activa = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SalaError::Validacion;

    fn extraer_errores(result: Result<Sala, SalaError>) -> Result<Vec<String>, String> {
        match result {
            Err(Validacion(errores)) => Ok(errores),
            Err(e) => Err(format!("Se esperaba Validacion, pero se obtuvo: {:?}", e)),
            Ok(s) => Err(format!("Se esperaba error, pero se obtuvo Ok({:?})", s)),
        }
    }

    fn assert_contiene_error(errores: &[String], claves: &[&str]) -> Result<(), String> {
        if errores.is_empty() {
            return Err("La lista de errores no debería estar vacía".into());
        }

        if errores.iter().any(|e| claves.iter().any(|k| e.contains(k))) {
            Ok(())
        } else {
            Err(format!(
                "Ningún mensaje contiene {:?}. Errores: {:?}",
                claves, errores
            ))
        }
    }

    #[test]
    fn crear_sala_valida() -> Result<(), String> {
        let sala = Sala::new("123".into(), "Sala de Conferencias".into(), 10)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert_eq!(sala.id(), "123");
        assert_eq!(sala.nombre(), "Sala de Conferencias");
        assert_eq!(sala.capacidad(), 10);
        assert!(sala.esta_activa());

        Ok(())
    }

    #[test]
    fn crear_sala_con_nombre_vacio() -> Result<(), String> {
        let errores = extraer_errores(Sala::new("123".into(), "".into(), 10))?;
        assert_contiene_error(&errores, &["nombre", "length"])
    }

    #[test]
    fn crear_sala_con_nombre_solo_espacios() -> Result<(), String> {
        let errores = extraer_errores(Sala::new("123".into(), "   ".into(), 10))?;
        assert_contiene_error(&errores, &["nombre", "length"])
    }

    #[test]
    fn crear_sala_con_nombre_demasiado_largo() -> Result<(), String> {
        let nombre_largo = "a".repeat(101);
        let errores = extraer_errores(Sala::new("123".into(), nombre_largo, 10))?;
        assert_contiene_error(&errores, &["nombre", "length"])
    }

    #[test]
    fn crear_sala_con_capacidad_cero() -> Result<(), String> {
        let errores = extraer_errores(Sala::new("123".into(), "Sala".into(), 0))?;
        assert_contiene_error(&errores, &["capacidad", "range", "Sobrepasa"])
    }

    #[test]
    fn crear_sala_con_capacidad_excesiva() -> Result<(), String> {
        let errores = extraer_errores(Sala::new("123".into(), "Sala".into(), 1001))?;
        assert_contiene_error(&errores, &["capacidad", "range", "Sobrepasa"])
    }

    #[test]
    fn activar_sala() -> Result<(), String> {
        let mut sala = Sala::new("123".into(), "Sala 1".into(), 10)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        sala.desactivar();
        assert!(!sala.esta_activa());

        sala.activar();
        assert!(sala.esta_activa());

        Ok(())
    }

    #[test]
    fn desactivar_sala() -> Result<(), String> {
        let mut sala = Sala::new("123".into(), "Sala 1".into(), 10)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert!(sala.esta_activa());

        sala.desactivar();
        assert!(!sala.esta_activa());

        Ok(())
    }

    #[test]
    fn nombre_trimea_espacios() -> Result<(), String> {
        let sala = Sala::new("123".into(), "  Sala con espacios  ".into(), 10)
            .map_err(|e| format!("No debería fallar: {:?}", e))?;

        assert_eq!(sala.nombre(), "Sala con espacios");
        Ok(())
    }
}

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Logger que escribe tanto a consola como a archivo
#[derive(Clone)]
pub struct Logger {
    file: Arc<Mutex<std::fs::File>>,
    log_path: PathBuf,
}

impl Logger {
    /// Crea un nuevo logger
    ///
    /// Busca la ruta del log en la variable de entorno LOG_FILE
    /// Si no está definida, usa /tmp/tauri-app.log por defecto
    pub fn new() -> Result<Self, std::io::Error> {
        let log_path = std::env::var("LOG_FILE")
            .unwrap_or_else(|_| "/tmp/tauri-app.log".to_string());

        Self::with_path(log_path)
    }

    /// Crea un logger con una ruta específica
    pub fn with_path(path: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let log_path = path.into();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            log_path,
        })
    }

    /// Retorna la ruta del archivo de log
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Escribe un mensaje de info
    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    /// Escribe un mensaje de error
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    /// Escribe un mensaje de debug
    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    /// Escribe un mensaje con un nivel específico
    fn log(&self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let formatted = format!("[{}] [{}] {}", timestamp, level, message);

        // Escribir a consola
        println!("{}", formatted);

        // Escribir a archivo
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file, "{}", formatted);
            let _ = file.flush();
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new().expect("Failed to create logger")
    }
}

// Macros para facilitar el uso
#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.debug(&format!($($arg)*))
    };
}

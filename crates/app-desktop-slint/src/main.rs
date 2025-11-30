slint::include_modules!();

use slint::{ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tonic::{metadata::MetadataValue, Request};

use salas_grpc::proto::{
    sala_service_client::SalaServiceClient, ActivarSalaRequest, CrearSalaRequest,
    DesactivarSalaRequest, ListarSalasRequest,
};
use usuarios_grpc::proto::{
    usuario_service_client::UsuarioServiceClient, LoginRequest,
};

const GRPC_URL: &str = "http://localhost:50051";

// Token JWT global (simple para demo)
lazy_static::lazy_static! {
    static ref JWT_TOKEN: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Modelo reactivo de salas
    let salas_model: Rc<VecModel<Sala>> = Rc::new(VecModel::default());
    ui.set_salas(ModelRc::from(salas_model.clone()));

    // Intentar login por defecto con credenciales de prueba
    // En una app real, tendrías una pantalla de login
    match login("admin@reservas.com", "admin123") {
        Ok(_) => {
            ui.set_mensaje("✅ Conectado al servidor gRPC".into());
            cargar_salas_ui(&ui, &salas_model);
        }
        Err(e) => {
            ui.set_mensaje(format!("⚠️ Login automático falló: {}. Funcionalidad limitada.", e).into());
        }
    }

    // Callback: Crear sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_crear_sala(move |nombre, capacidad| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);
            ui.set_mensaje("".into());

            if nombre.is_empty() {
                ui.set_mensaje("❌ El nombre no puede estar vacío".into());
                ui.set_loading(false);
                return;
            }

            if capacidad <= 0 {
                ui.set_mensaje("❌ La capacidad debe ser mayor que 0".into());
                ui.set_loading(false);
                return;
            }

            match crear_sala(&nombre.to_string(), capacidad as u32) {
                Ok(_) => {
                    ui.set_mensaje(format!("✅ Sala '{}' creada correctamente", nombre).into());
                    ui.set_nuevo_nombre("".into());
                    ui.set_nueva_capacidad(10);

                    // Recargar salas
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al crear sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Activar sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_activar_sala(move |id| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);

            match activar_sala(&id.to_string()) {
                Ok(_) => {
                    ui.set_mensaje("✅ Sala activada correctamente".into());
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al activar sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Desactivar sala
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_desactivar_sala(move |id| {
            let ui = ui_weak.unwrap();
            ui.set_loading(true);

            match desactivar_sala(&id.to_string()) {
                Ok(_) => {
                    ui.set_mensaje("✅ Sala desactivada correctamente".into());
                    cargar_salas_ui(&ui, &salas_model);
                }
                Err(e) => {
                    ui.set_mensaje(format!("❌ Error al desactivar sala: {}", e).into());
                }
            }

            ui.set_loading(false);
        });
    }

    // Callback: Cargar salas
    {
        let ui_weak = ui.as_weak();
        let salas_model = salas_model.clone();

        ui.on_cargar_salas(move || {
            let ui = ui_weak.unwrap();
            cargar_salas_ui(&ui, &salas_model);
        });
    }

    ui.run()
}

fn cargar_salas_ui(ui: &AppWindow, model: &Rc<VecModel<Sala>>) {
    match listar_salas() {
        Ok(salas) => {
            model.set_vec(
                salas
                    .into_iter()
                    .map(|s| Sala {
                        id: s.id.into(),
                        nombre: s.nombre.into(),
                        capacidad: s.capacidad as i32,
                        activa: s.activa,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        Err(e) => {
            ui.set_mensaje(format!("❌ Error al cargar salas: {}", e).into());
        }
    }
}

// Helper para añadir token JWT a la request
fn add_auth_token<T>(request: &mut Request<T>) -> Result<(), String> {
    let token = JWT_TOKEN
        .lock()
        .map_err(|e| format!("Error al acceder al token: {}", e))?
        .clone()
        .ok_or_else(|| "No hay token de autenticación disponible".to_string())?;

    let auth_value = MetadataValue::try_from(format!("Bearer {}", token))
        .map_err(|e| format!("Error al crear header de autorización: {}", e))?;

    request.metadata_mut().insert("authorization", auth_value);
    Ok(())
}

// API functions usando gRPC con tokio runtime
fn login(email: &str, password: &str) -> Result<String, String> {
    let rt = Runtime::new().map_err(|e| format!("Error al crear runtime: {}", e))?;

    rt.block_on(async {
        let mut client = UsuarioServiceClient::connect(GRPC_URL)
            .await
            .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

        let request = Request::new(LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = client
            .login(request)
            .await
            .map_err(|e| format!("Error de login: {}", e))?;

        let token = response.into_inner().token;

        // Guardar token
        JWT_TOKEN
            .lock()
            .map_err(|e| format!("Error al guardar token: {}", e))?
            .replace(token.clone());

        Ok(token)
    })
}

fn listar_salas() -> Result<Vec<SalaDto>, String> {
    let rt = Runtime::new().map_err(|e| format!("Error al crear runtime: {}", e))?;

    rt.block_on(async {
        let mut client = SalaServiceClient::connect(GRPC_URL)
            .await
            .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

        let mut request = Request::new(ListarSalasRequest {});
        add_auth_token(&mut request)?;

        let response = client
            .listar_salas(request)
            .await
            .map_err(|e| format!("Error gRPC: {}", e))?;

        let salas = response
            .into_inner()
            .salas
            .into_iter()
            .map(|s| SalaDto {
                id: s.id,
                nombre: s.nombre,
                capacidad: s.capacidad,
                activa: s.activa,
            })
            .collect();

        Ok(salas)
    })
}

fn crear_sala(nombre: &str, capacidad: u32) -> Result<SalaDto, String> {
    let rt = Runtime::new().map_err(|e| format!("Error al crear runtime: {}", e))?;

    rt.block_on(async {
        let mut client = SalaServiceClient::connect(GRPC_URL)
            .await
            .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

        let mut request = Request::new(CrearSalaRequest {
            nombre: nombre.to_string(),
            capacidad,
        });
        add_auth_token(&mut request)?;

        let response = client
            .crear_sala(request)
            .await
            .map_err(|e| format!("Error gRPC: {}", e))?;

        let sala = response.into_inner();
        Ok(SalaDto {
            id: sala.id,
            nombre: sala.nombre,
            capacidad: sala.capacidad,
            activa: sala.activa,
        })
    })
}

fn activar_sala(id: &str) -> Result<SalaDto, String> {
    let rt = Runtime::new().map_err(|e| format!("Error al crear runtime: {}", e))?;

    rt.block_on(async {
        let mut client = SalaServiceClient::connect(GRPC_URL)
            .await
            .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

        let mut request = Request::new(ActivarSalaRequest {
            id: id.to_string(),
        });
        add_auth_token(&mut request)?;

        let response = client
            .activar_sala(request)
            .await
            .map_err(|e| format!("Error gRPC: {}", e))?;

        let sala = response.into_inner();
        Ok(SalaDto {
            id: sala.id,
            nombre: sala.nombre,
            capacidad: sala.capacidad,
            activa: sala.activa,
        })
    })
}

fn desactivar_sala(id: &str) -> Result<SalaDto, String> {
    let rt = Runtime::new().map_err(|e| format!("Error al crear runtime: {}", e))?;

    rt.block_on(async {
        let mut client = SalaServiceClient::connect(GRPC_URL)
            .await
            .map_err(|e| format!("Error de conexión gRPC: {}", e))?;

        let mut request = Request::new(DesactivarSalaRequest {
            id: id.to_string(),
        });
        add_auth_token(&mut request)?;

        let response = client
            .desactivar_sala(request)
            .await
            .map_err(|e| format!("Error gRPC: {}", e))?;

        let sala = response.into_inner();
        Ok(SalaDto {
            id: sala.id,
            nombre: sala.nombre,
            capacidad: sala.capacidad,
            activa: sala.activa,
        })
    })
}

#[derive(Debug, Clone)]
struct SalaDto {
    id: String,
    nombre: String,
    capacidad: u32,
    activa: bool,
}

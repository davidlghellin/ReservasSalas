use dioxus::prelude::*;

use crate::models::{AppState, UsuarioInfo};
use crate::services::login_usuario;

#[component]
pub fn LoginScreen(
    mut app_state: Signal<AppState>,
    mut token: Signal<Option<String>>,
    mut usuario_actual: Signal<Option<UsuarioInfo>>,
) -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| false);

    let login_handler = move |_| {
        spawn(async move {
            let email_val = email.read().clone();
            let password_val = password.read().clone();

            if email_val.is_empty() || password_val.is_empty() {
                error.set("Email y contraseña son requeridos".to_string());
                return;
            }

            loading.set(true);
            error.set(String::new());

            match login_usuario(&email_val, &password_val).await {
                Ok((usuario, tok)) => {
                    loading.set(false);
                    token.set(Some(tok));
                    usuario_actual.set(Some(usuario.clone()));
                    app_state.set(AppState::Authenticated(usuario));
                }
                Err(e) => {
                    loading.set(false);
                    error.set(e);
                }
            }
        });
    };

    rsx! {
        style { {include_str!("../../assets/style.css")} }

        div { class: "login-container",
            div { class: "login-box",
                h1 { class: "login-title", "🔐 Iniciar Sesión" }
                p { class: "login-subtitle", "Sistema de Gestión de Salas" }

                if !error.read().is_empty() {
                    div { class: "error-message",
                        "{error}"
                    }
                }

                form { class: "login-form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        login_handler(());
                    },

                    div { class: "form-group",
                        label { r#for: "email", "Email:" }
                        input {
                            id: "email",
                            r#type: "email",
                            placeholder: "usuario@ejemplo.com",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "password", "Contraseña:" }
                        input {
                            id: "password",
                            r#type: "password",
                            placeholder: "••••••••",
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                            disabled: *loading.read(),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "btn btn-primary btn-block",
                        disabled: *loading.read(),
                        if *loading.read() {
                            "⏳ Iniciando sesión..."
                        } else {
                            "🚀 Iniciar Sesión"
                        }
                    }
                }
            }
        }
    }
}

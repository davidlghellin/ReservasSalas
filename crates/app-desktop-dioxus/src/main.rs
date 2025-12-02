#![allow(non_snake_case)]

mod calendario;
mod components;
mod models;
mod services;

use dioxus::prelude::*;

use components::{LoginScreen, SalasApp};
use models::{AppState, UsuarioInfo};

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app_state = use_signal(|| AppState::Login);
    let token = use_signal(|| Option::<String>::None);
    let usuario_actual = use_signal(|| Option::<UsuarioInfo>::None);

    let current_state = app_state.read().clone();

    match current_state {
        AppState::Login => {
            rsx! {
                LoginScreen {
                    app_state: app_state,
                    token: token,
                    usuario_actual: usuario_actual,
                }
            }
        }
        AppState::Authenticated(_) => {
            let usuario = usuario_actual.read().clone();
            if let Some(usuario) = usuario {
                rsx! {
                    SalasApp {
                        usuario: usuario.clone(),
                        token: token,
                        app_state: app_state,
                        usuario_actual: usuario_actual,
                    }
                }
            } else {
                rsx! { div { "Error: Usuario no encontrado" } }
            }
        }
    }
}

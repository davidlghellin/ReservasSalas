use dioxus::prelude::*;
use reservas_grpc::proto::Reserva as ProtoReserva;
use salas_grpc::proto::SalaResponse;
use chrono::{DateTime, Datelike, Duration, Local, Timelike};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VistaCalendario {
    Diaria,
    Semanal,
}

#[component]
pub fn CalendarioDiario(
    reservas: Vec<ProtoReserva>,
    salas: Vec<SalaResponse>,
    fecha: DateTime<Local>,
) -> Element {
    // Franjas horarias de 8:00 a 20:00
    let horas = (8..20).collect::<Vec<_>>();

    // Filtrar reservas del día actual
    let reservas_del_dia: Vec<_> = reservas
        .iter()
        .filter(|r| {
            if let Ok(inicio) = DateTime::parse_from_rfc3339(&r.fecha_inicio) {
                let inicio_local = inicio.with_timezone(&Local);
                inicio_local.date_naive() == fecha.date_naive()
            } else {
                false
            }
        })
        .collect();

    rsx! {
        div { class: "calendario-diario",
            // Header con la fecha
            div { class: "calendario-header",
                h2 { "📅 Vista Diaria - {fecha.format(\"%d/%m/%Y\")}" }
            }

            // Grid de franjas horarias
            div { class: "franjas-horarias",
                for hora in horas.iter() {
                    div { class: "franja-horaria",
                        div { class: "hora-label",
                            "{hora:02}:00"
                        }

                        // Mostrar salas para esta hora
                        div { class: "salas-en-hora",
                            for sala in salas.iter() {
                                {
                                    let reserva_en_hora = reservas_del_dia.iter().find(|r| {
                                        if let (Ok(inicio), Ok(fin)) = (
                                            DateTime::parse_from_rfc3339(&r.fecha_inicio),
                                            DateTime::parse_from_rfc3339(&r.fecha_fin)
                                        ) {
                                            let inicio_local = inicio.with_timezone(&Local);
                                            let fin_local = fin.with_timezone(&Local);

                                            let hora_inicio = inicio_local.hour() as i32;
                                            let hora_fin = fin_local.hour() as i32;

                                            r.sala_id == sala.id && hora_inicio <= *hora && *hora < hora_fin
                                        } else {
                                            false
                                        }
                                    });

                                    if let Some(reserva) = reserva_en_hora {
                                        let estado_class = match reserva.estado {
                                            0 => "reserva-activa",
                                            1 => "reserva-cancelada",
                                            2 => "reserva-completada",
                                            _ => "reserva-desconocida",
                                        };

                                        rsx! {
                                            div { class: "sala-slot {estado_class}",
                                                div { class: "sala-nombre", "{sala.nombre}" }
                                                div { class: "reserva-info",
                                                    "🕐 {format_hora_reserva(&reserva.fecha_inicio, &reserva.fecha_fin)}"
                                                }
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            div { class: "sala-slot libre",
                                                div { class: "sala-nombre", "{sala.nombre}" }
                                                div { class: "disponible", "✅ Disponible" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Leyenda
            div { class: "leyenda",
                div { class: "leyenda-item",
                    div { class: "color-box libre" }
                    "Disponible"
                }
                div { class: "leyenda-item",
                    div { class: "color-box reserva-activa" }
                    "Reservada"
                }
                div { class: "leyenda-item",
                    div { class: "color-box reserva-cancelada" }
                    "Cancelada"
                }
            }
        }
    }
}

#[component]
pub fn CalendarioSemanal(
    reservas: Vec<ProtoReserva>,
    salas: Vec<SalaResponse>,
    fecha_inicio: DateTime<Local>,
) -> Element {
    // Generar los 7 días de la semana
    let dias: Vec<DateTime<Local>> = (0..7)
        .map(|i| fecha_inicio + Duration::days(i))
        .collect();

    rsx! {
        div { class: "calendario-semanal",
            // Header
            div { class: "calendario-header",
                h2 {
                    "📅 Vista Semanal - Semana del {fecha_inicio.format(\"%d/%m/%Y\")}"
                }
            }

            // Grid semanal
            div { class: "grid-semanal",
                // Header con los días
                div { class: "dias-header",
                    div { class: "sala-column-header", "Sala" }
                    for dia in dias.iter() {
                        div { class: "dia-header",
                            div { class: "dia-nombre", "{get_nombre_dia(dia.weekday())}" }
                            div { class: "dia-fecha", "{dia.format(\"%d/%m\")}" }
                        }
                    }
                }

                // Filas por sala
                for sala in salas.iter() {
                    div { class: "fila-sala",
                        div { class: "sala-nombre-col",
                            "🏢 {sala.nombre}"
                            br {}
                            span { class: "capacidad-text", "👥 {sala.capacidad}" }
                        }

                        // Columnas por día
                        for dia in dias.iter() {
                            {
                                let reservas_del_dia = get_reservas_sala_dia(
                                    &reservas,
                                    &sala.id,
                                    *dia
                                );

                                rsx! {
                                    div { class: "celda-dia",
                                        if reservas_del_dia.is_empty() {
                                            div { class: "sin-reservas", "✅ Libre" }
                                        } else {
                                            div { class: "reservas-dia",
                                                for reserva in reservas_del_dia.iter() {
                                                    {
                                                        let estado_emoji = match reserva.estado {
                                                            0 => "📅",
                                                            1 => "❌",
                                                            2 => "✅",
                                                            _ => "❓",
                                                        };

                                                        let hora_info = format_hora_reserva(
                                                            &reserva.fecha_inicio,
                                                            &reserva.fecha_fin
                                                        );

                                                        rsx! {
                                                            div { class: "reserva-item",
                                                                "{estado_emoji} {hora_info}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Leyenda
            div { class: "leyenda",
                div { class: "leyenda-item",
                    "📅 Activa"
                }
                div { class: "leyenda-item",
                    "❌ Cancelada"
                }
                div { class: "leyenda-item",
                    "✅ Completada"
                }
            }
        }
    }
}

// Helpers
fn format_hora_reserva(inicio: &str, fin: &str) -> String {
    if let (Ok(inicio_dt), Ok(fin_dt)) = (
        DateTime::parse_from_rfc3339(inicio),
        DateTime::parse_from_rfc3339(fin),
    ) {
        let inicio_local = inicio_dt.with_timezone(&Local);
        let fin_local = fin_dt.with_timezone(&Local);
        format!(
            "{:02}:{:02}-{:02}:{:02}",
            inicio_local.hour(),
            inicio_local.minute(),
            fin_local.hour(),
            fin_local.minute()
        )
    } else {
        "??:??".to_string()
    }
}

fn get_reservas_sala_dia(
    reservas: &[ProtoReserva],
    sala_id: &str,
    dia: DateTime<Local>,
) -> Vec<ProtoReserva> {
    reservas
        .iter()
        .filter(|r| {
            if let Ok(inicio) = DateTime::parse_from_rfc3339(&r.fecha_inicio) {
                let inicio_local = inicio.with_timezone(&Local);
                r.sala_id == sala_id && inicio_local.date_naive() == dia.date_naive()
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

fn get_nombre_dia(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Lun",
        chrono::Weekday::Tue => "Mar",
        chrono::Weekday::Wed => "Mié",
        chrono::Weekday::Thu => "Jue",
        chrono::Weekday::Fri => "Vie",
        chrono::Weekday::Sat => "Sáb",
        chrono::Weekday::Sun => "Dom",
    }
}

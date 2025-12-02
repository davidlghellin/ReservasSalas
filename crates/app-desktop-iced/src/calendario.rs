use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use iced::widget::{column, container, row, scrollable, text, Column, Row};
use iced::{Alignment, Element, Length};
use reservas_grpc::proto::Reserva as ProtoReserva;
use salas_grpc::proto::SalaResponse;

use crate::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VistaCalendario {
    Diaria,
    Semanal,
}

pub fn view_calendario_diario(
    reservas: Vec<ProtoReserva>,
    salas: Vec<SalaResponse>,
    fecha: DateTime<Local>,
) -> Element<'static, Message> {
    // Franjas horarias de 8:00 a 20:00
    let horas: Vec<i32> = (8..20).collect();

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

    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .width(Length::Fill);

    // Header
    content = content.push(
        text(format!("📅 Vista Diaria - {}", fecha.format("%d/%m/%Y")))
            .size(24)
            .width(Length::Fill),
    );

    // Grid de franjas horarias
    for hora in horas.iter() {
        let mut franja_row = Row::new()
            .spacing(10)
            .align_y(Alignment::Start)
            .padding(5);

        // Label de hora
        franja_row = franja_row.push(
            container(
                text(format!("{:02}:00", hora))
                    .size(16)
                    .width(Length::Fill),
            )
            .width(Length::Fixed(80.0))
            .padding(10),
        );

        // Salas para esta hora
        let mut salas_row = Row::new().spacing(5).width(Length::Fill);

        for sala in salas.iter() {
            let reserva_en_hora = reservas_del_dia.iter().find(|r| {
                if let (Ok(inicio), Ok(fin)) = (
                    DateTime::parse_from_rfc3339(&r.fecha_inicio),
                    DateTime::parse_from_rfc3339(&r.fecha_fin),
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

            let sala_nombre = sala.nombre.clone();
            let sala_content = if let Some(reserva) = reserva_en_hora {
                let estado_text = match reserva.estado {
                    0 => "📅 Reservada",
                    1 => "❌ Cancelada",
                    2 => "✅ Completada",
                    _ => "❓ Desconocida",
                };

                let horario = format_hora_reserva(&reserva.fecha_inicio, &reserva.fecha_fin);

                column![
                    text(sala_nombre).size(14),
                    text(estado_text).size(12),
                    text(format!("🕐 {}", horario)).size(11),
                ]
                .spacing(3)
                .padding(8)
            } else {
                column![
                    text(sala_nombre).size(14),
                    text("✅ Disponible").size(12),
                ]
                .spacing(3)
                .padding(8)
            };

            salas_row = salas_row.push(container(sala_content).padding(5).width(Length::Fill));
        }

        franja_row = franja_row.push(salas_row);
        content = content.push(container(franja_row).padding(5).width(Length::Fill));
    }

    // Leyenda
    let leyenda = row![
        text("Leyenda: ").size(14),
        text("✅ Disponible").size(14),
        text("📅 Reservada").size(14),
        text("❌ Cancelada").size(14),
    ]
    .spacing(15)
    .padding(15);

    content = content.push(leyenda);

    scrollable(content)
        .width(Length::Fill)
        .into()
}

pub fn view_calendario_semanal(
    reservas: Vec<ProtoReserva>,
    salas: Vec<SalaResponse>,
    fecha_inicio: DateTime<Local>,
) -> Element<'static, Message> {
    // Generar los 7 días de la semana
    let dias: Vec<DateTime<Local>> = (0..7).map(|i| fecha_inicio + Duration::days(i)).collect();

    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .width(Length::Fill);

    // Header
    content = content.push(
        text(format!(
            "📅 Vista Semanal - Semana del {}",
            fecha_inicio.format("%d/%m/%Y")
        ))
        .size(24)
        .width(Length::Fill),
    );

    // Header con los días
    let mut header_row = Row::new().spacing(5).padding(5);
    header_row = header_row.push(
        container(text("Sala").size(14))
            .width(Length::Fixed(150.0))
            .padding(10),
    );

    for dia in dias.iter() {
        let dia_text = format!(
            "{}\n{}",
            get_nombre_dia(dia.weekday()),
            dia.format("%d/%m")
        );
        header_row = header_row.push(
            container(text(dia_text).size(12))
                .width(Length::FillPortion(1))
                .padding(8),
        );
    }
    content = content.push(header_row);

    // Filas por sala
    for sala in salas.iter() {
        let mut fila_row = Row::new().spacing(5).padding(5);

        // Nombre de sala
        fila_row = fila_row.push(
            container(
                column![
                    text(format!("🏢 {}", sala.nombre)).size(14),
                    text(format!("👥 {}", sala.capacidad)).size(11),
                ]
                .spacing(3),
            )
            .width(Length::Fixed(150.0))
            .padding(10),
        );

        // Columnas por día
        for dia in dias.iter() {
            let reservas_del_dia = get_reservas_sala_dia(&reservas, &sala.id, *dia);

            let celda_content = if reservas_del_dia.is_empty() {
                column![text("✅ Libre").size(12)]
                    .spacing(3)
                    .padding(8)
            } else {
                let mut col = Column::new().spacing(3).padding(8);
                for reserva in reservas_del_dia.iter() {
                    let estado_emoji = match reserva.estado {
                        0 => "📅",
                        1 => "❌",
                        2 => "✅",
                        _ => "❓",
                    };

                    let hora_info = format_hora_reserva(&reserva.fecha_inicio, &reserva.fecha_fin);

                    col = col.push(text(format!("{} {}", estado_emoji, hora_info)).size(11));
                }
                col
            };

            fila_row = fila_row.push(
                container(celda_content)
                    .width(Length::FillPortion(1))
                    .padding(5),
            );
        }

        content = content.push(container(fila_row).padding(5).width(Length::Fill));
    }

    // Leyenda
    let leyenda = row![
        text("Leyenda: ").size(14),
        text("📅 Activa").size(14),
        text("❌ Cancelada").size(14),
        text("✅ Completada").size(14),
    ]
    .spacing(15)
    .padding(15);

    content = content.push(leyenda);

    scrollable(content)
        .width(Length::Fill)
        .into()
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
    reservas: &Vec<ProtoReserva>,
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

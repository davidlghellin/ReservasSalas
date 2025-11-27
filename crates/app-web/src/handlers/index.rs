use crate::templates::IndexTemplate;
use askama::Template;
use axum::response::{Html, IntoResponse};

pub async fn index() -> impl IntoResponse {
    Html(IndexTemplate.render().unwrap())
}

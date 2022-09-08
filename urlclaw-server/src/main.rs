use askama::Template;
use axum::{
    extract::{Extension, Form, Path},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use urlclaw_core::repository::memory::InMemoryRepository;
use urlclaw_core::repository::RepositoryError;
use urlclaw_core::service;

#[tokio::main]
async fn main() {
    let bind = "127.0.0.1:8000";

    println!("Listening on http://{bind}");

    let mem_repo = Arc::new(Mutex::new(InMemoryRepository::default()));

    let router = Router::new()
        .route("/", get(index))
        .route("/", post(create_shorturl))
        .route("/:path", get(handle_path))
        .layer(Extension(mem_repo));

    axum::Server::bind(&bind.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> impl IntoResponse {
    let index = IndexTemplate {};

    Html(index.render().unwrap())
}

#[derive(Template)]
#[template(path = "created.html")]
struct CreatedShortUrlTemplate {
    short: String,
    target: String,
}

#[derive(Deserialize)]
struct CreateShortUrlIn {
    short: String,
    target: String,
}

async fn create_shorturl(
    Form(data): Form<CreateShortUrlIn>,
    Extension(repo): Extension<Arc<Mutex<InMemoryRepository>>>,
) -> impl IntoResponse {
    let short_url = service::create_shorturl(
        &mut *repo.lock().await,
        data.short.clone(),
        data.target.clone(),
    )
    .await;

    match short_url {
        Ok(short_url) => {
            let template = CreatedShortUrlTemplate {
                short: short_url.short,
                target: short_url.target,
            };
            Html(template.render().unwrap()).into_response()
        }
        Err(service::ServiceError::Repository(RepositoryError::AlreadyExists)) => {
            format!("Sorry, short already exists.").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("fail: {e:?}")).into_response(),
    }
}

async fn handle_path(
    Path(path): Path<String>,
    Extension(repo): Extension<Arc<Mutex<InMemoryRepository>>>,
) -> impl IntoResponse {
    let short_url = service::get_shorturl_target(&mut *repo.lock().await, &path).await;

    match short_url {
        Ok(short_url) => (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, short_url.target.clone())],
            format!("Path: {short_url:?}"),
        )
            .into_response(),
        Err(service::ServiceError::Repository(RepositoryError::NoUrlFound)) => {
            (StatusCode::NOT_FOUND, format!("short not found")).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, format!("fail")).into_response(),
    }
}

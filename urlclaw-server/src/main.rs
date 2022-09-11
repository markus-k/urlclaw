use askama::Template;
use axum::{
    extract::{Extension, Form, Path},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use urlclaw_core::service;
use urlclaw_core::{repository::sqlx::SqlxRepository, UrlclawError};

type SharedRepo = SqlxRepository;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").map_or(80, |p| p.parse().expect("can't parse PORT"));
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let bind = format!("{bind_addr}:{port}");

    println!("Listening on http://{bind}");

    //let mem_repo = Arc::new(Mutex::new(InMemoryRepository::default()));

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());
    let sqlx_repo = SqlxRepository::new(&database_url).await.unwrap();
    sqlx_repo.migrate().await.unwrap();

    let repo = sqlx_repo; // no need to Arc and mutex here

    let router = Router::new()
        .route("/", get(index))
        .route("/", post(create_shorturl))
        .route("/:path", get(handle_path))
        .layer(Extension(repo));

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
    Extension(repo): Extension<SharedRepo>,
) -> impl IntoResponse {
    let short_url =
        service::create_shorturl(&mut repo.clone(), data.short.clone(), data.target.clone()).await;

    match short_url {
        Ok(short_url) => {
            let template = CreatedShortUrlTemplate {
                short: short_url.short_url().as_str().to_owned(),
                target: short_url.target_url().to_string(),
            };
            Html(template.render().unwrap()).into_response()
        }
        Err(UrlclawError::ShortAlreadyExists) => "Sorry, short already exists.".into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("fail: {e:?}")).into_response(),
    }
}

async fn handle_path(
    Path(path): Path<String>,
    Extension(repo): Extension<SharedRepo>,
) -> impl IntoResponse {
    let short_url = service::get_shorturl_target(&mut repo.clone(), &path).await;

    match short_url {
        Ok(short_url) => (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, short_url.target_url().to_string())],
            format!("Redirecting to {}", short_url.target_url()),
        )
            .into_response(),
        Err(UrlclawError::UrlNotFound) => {
            (StatusCode::NOT_FOUND, "short not found").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("fail: {e:?}")).into_response(),
    }
}

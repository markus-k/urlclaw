use axum::{
    async_trait,
    extract::{FromRequest, Query, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, Expiration, Key, SameSite},
    SignedCookieJar,
};
use openidconnect::{
    core::CoreAuthenticationFlow,
    core::{CoreClient, CoreIdToken, CoreProviderMetadata},
    reqwest::async_http_client,
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, RedirectUrl, Scope, TokenResponse,
};
use serde::Deserialize;
use url::Url;

pub async fn make_auth_router() -> Router {
    let core_client = make_coreclient().await;

    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
        .layer(Extension(core_client))
}

async fn make_coreclient() -> CoreClient {
    let oauth_domain = std::env::var("OAUTH_DOMAIN").expect("OAUTH_DOMAIN is not set");
    let client_id = std::env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID is not set");
    let client_secret =
        std::env::var("OAUTH_CLIENT_SECRET").expect("OAUTH_CLIENT_SECRET is not set");
    let callback_url = std::env::var("OAUTH_CALLBACK_URL").expect("OAUTH_CALLBACK_URL is not set");

    let provider = CoreProviderMetadata::discover_async(
        IssuerUrl::new(format!("https://{oauth_domain}/")).unwrap(),
        async_http_client,
    )
    .await
    .unwrap();

    CoreClient::from_provider_metadata(
        provider,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    )
    .set_redirect_uri(RedirectUrl::new(callback_url).unwrap())
}

async fn login(
    Extension(client): Extension<CoreClient>,
    cookie_jar: SignedCookieJar,
) -> impl IntoResponse {
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_owned()))
        .add_scope(Scope::new("email".to_owned()))
        .add_scope(Scope::new("profile".to_owned()))
        .url();

    (
        cookie_jar
            .add(
                Cookie::build("auth_csrf_token", csrf_token.secret().clone())
                    .http_only(true)
                    .finish(),
            )
            .add(
                Cookie::build("auth_nonce", nonce.secret().clone())
                    .http_only(true)
                    .path("/")
                    .finish(),
            ),
        Redirect::temporary(auth_url.as_str()),
    )
        .into_response()
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn callback(
    Query(CallbackQuery { code, state }): Query<CallbackQuery>,
    Extension(client): Extension<CoreClient>,
    cookie_jar: SignedCookieJar,
) -> impl IntoResponse {
    let csrf_token_cookie = match cookie_jar.get("auth_csrf_token") {
        Some(csrf_token) => csrf_token,
        None => return (StatusCode::BAD_REQUEST, "missing auth_csrf_token").into_response(),
    };
    let csrf_token = csrf_token_cookie.value().to_owned();
    let nonce_cookie = match cookie_jar.get("auth_nonce") {
        Some(nonce) => nonce,
        None => return (StatusCode::BAD_REQUEST, "missing auth_nonce").into_response(),
    };
    let nonce = Nonce::new(nonce_cookie.value().to_owned());

    if csrf_token != state {
        return (StatusCode::BAD_REQUEST, "invalid csrf_token").into_response();
    }

    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .unwrap();

    let id_token = match token_response.id_token() {
        Some(id_token) => id_token,
        None => {
            return (StatusCode::UNAUTHORIZED, "no id-token received from server").into_response();
        }
    };
    let claims = id_token
        .claims(&client.id_token_verifier(), &nonce)
        .unwrap();

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            &id_token.signing_alg().unwrap(),
        )
        .unwrap();
        if actual_access_token_hash != *expected_access_token_hash {
            return (StatusCode::UNAUTHORIZED, "Invalid access token").into_response();
        }
    }

    (
        cookie_jar
            .remove(csrf_token_cookie)
            //.remove(nonce_cookie)
            .add(
                Cookie::build("id_token", id_token.to_string())
                    .http_only(true)
                    .path("/")
                    .expires(Expiration::DateTime(
                        time::OffsetDateTime::from_unix_timestamp(claims.expiration().timestamp())
                            .unwrap(),
                    ))
                    .same_site(SameSite::Strict)
                    .finish(),
            ),
        format!(
            "Hi {}\n\nsecret: {}\n\nid token: {}",
            claims.subject().as_str(),
            token_response.access_token().secret(),
            id_token.to_string(),
        ),
    )
        .into_response()
}

async fn logout(cookie_jar: SignedCookieJar) -> impl IntoResponse {
    let logout_return_url =
        std::env::var("OAUTH_LOGOUT_RETURN_URL").expect("OAUTH_LOGOUT_RETURN_URL is not set");
    let access_token_cookie = match cookie_jar.get("access_token") {
        Some(cookie) => cookie,
        None => return Redirect::temporary(&logout_return_url).into_response(),
    };

    let oauth_domain = std::env::var("OAUTH_DOMAIN").expect("OAUTH_DOMAIN is not set");
    let client_id = std::env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID is not set");

    let mut logout_url = Url::parse(&format!("https://{oauth_domain}/v2/logout")).unwrap();
    logout_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("returnTo", &logout_return_url);

    (
        cookie_jar.remove(access_token_cookie),
        Redirect::temporary(&logout_url.to_string()),
    )
        .into_response()
}

pub struct Authenticated {
    pub user_id: String,
}

#[async_trait]
impl<B> FromRequest<B> for Authenticated
where
    B: Send,
{
    type Rejection = Response;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let cookie_jar = SignedCookieJar::<Key>::from_request(req).await.unwrap();
        let client = make_coreclient().await;

        if let Some(id_token_cookie) = cookie_jar.get("id_token") {
            let id_token: CoreIdToken = id_token_cookie.value().parse().unwrap();

            if let Some(nonce_cookie) = cookie_jar.get("auth_nonce") {
                let nonce = Nonce::new(nonce_cookie.value().to_owned());
                let claims = id_token
                    .claims(&client.id_token_verifier(), &nonce)
                    .unwrap();

                Ok(Authenticated {
                    user_id: claims.subject().to_string(),
                })
            } else {
                Err(Redirect::to("/auth/login").into_response())
            }
        } else {
            Err(Redirect::to("/auth/login").into_response())
        }
    }
}

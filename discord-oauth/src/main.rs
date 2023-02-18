/*
This crate provides a generic interface between cookie values and storage backends to create a concept of sessions. It provides an interface that can be used to encode and store sessions, and decode and load sessions generating cookies in the process.
 */
use async_session::{MemoryStore, Session, SessionStore};

use axum::{
    async_trait,
    extract::{
        rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, Query, State, TypedHeader,
    },
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    RequestPartsExt, Router,
};
use dotenv::dotenv;

use http::{header, request::Parts};

// https://docs.rs/oauth2/latest/oauth2/
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};

use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};

use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

static COOKIE_NAME: &str = "SESSION";
#[tokio::main]
async fn main() {
    dotenv().ok();
    /*
    tracing is a framework for instrumenting Rust programs to collect scoped, structured, and async-aware diagnostics. The Subscriber trait represents the functionality necessary to collect this trace data. This crate contains tools for composing subscribers out of smaller units of behaviour, and batteries-included implementations of common subscriber functionality.

    tracing-subscriber is intended for use by both Subscriber authors and application authors using tracing to instrument their applications.

    The most important component of the tracing-subscriber API is the Layer trait, which provides a composable abstraction for building Subscribers. Like the Subscriber trait, a Layer defines a particular behavior for collecting trace data. Unlike Subscribers, which implement a complete strategy for how trace data is collected,
    Layers provide modular implementations of specific behaviors. Therefore, they can be composed together to form a Subscriber which is capable of recording traces in a variety of ways.

    https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html
     */
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_oauth=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // MemoryStore` is just used as an example. Don't use this in production.

    let store = MemoryStore::new();
    let oauth_client = oauth_client();
    let app_state = AppState {
        store,
        oauth_client,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/discord", get(discord_auth))
        .route("/auth/authorized", get(login_authorized))
        .route("/protected", get(protected))
        .route("/logout", get(logout))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {} ", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
struct AppState {
    store: MemoryStore,
    oauth_client: BasicClient,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(input: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(input: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

fn oauth_client() -> BasicClient {
    let client_id = env::var("CLIENT_ID").expect("Missing CLIENT_ID");
    let client_secret = env::var("CLIENT_SECRET").expect("Misssing CLIENT_SECRET");
    let redirect_url = env::var("REDIRECT_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000/auth/authorized".to_string());
    let auth_url = env::var("AUTH_URL").unwrap_or_else(|_| {
        "https://discord.com/api/oauth2/authorize?response_type=code".to_string()
    });

    let token_url = env::var("TOKEN_URL")
        .unwrap_or_else(|_| "https://discord.com/api/oauth2/token".to_string());

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).unwrap(),
        Some(TokenUrl::new(token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

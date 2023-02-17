use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {} ", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Result<(), AppError> {
    try_thing()?;
    Ok(())
}

fn try_thing() -> Result<(), anyhow::Error> {
    anyhow::bail!("it failed!")
}

//Make our own error that wraps 'anyhow::Error'
struct AppError(anyhow::Error);

// Tell Axum how to convert 'AppError' into a response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Something went wrong: {}", self.0),
        )
        .into_response()
    }
   
}


// OUTPUT - Something went wrong: it failed!
// This enables using '?' on functions that return 'Result<_, anyhow::Error>' to turn them into
// Result<_, AppError>. That way you don't need to do that manually
impl<E> From <E> for AppError
where
E: Into<anyhow::Error>,
{
    fn from(err:E) -> Self {
        Self(err.into())
    }
}
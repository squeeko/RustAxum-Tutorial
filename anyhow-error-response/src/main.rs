use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
/*
Trait for generating responses.

Types that implement IntoResponse can be returned from handlers.
You generally shouldn’t have to implement IntoResponse manually, as axum provides implementations for many common types.

However it might be necessary if you have a custom error type that you want to return from handlers like shown below
*/
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
// Convert this router into a MakeService, that is a Service whose response is another service.
/*
Creates new Service values.

Acts as a service factory. This is useful for cases where new Service values must be produced. One case is a TCP server listener. The listener accepts new TCP streams, obtains a new Service value using the MakeService trait, and uses that new Service value to process inbound requests on that new TCP stream.

This is essentially a trait alias for a Service of Services.
*/
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
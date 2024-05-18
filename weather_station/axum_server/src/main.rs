//! Run with
//!
//! ```not_rust
//! cargo watch -x run
//! ```

use weather_station::actors::db_actor::{DbActorHandle, Reading};


use axum::http::{Response, StatusCode};
use axum::{routing::get, routing::post, Router};
use axum::{Error, Extension};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, span, Level};



#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        // Use compact abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line number
        .with_line_number(true)
        // Dislpay the thread ID the event was recorded on
        .with_thread_ids(true)
        // Don't display events target (module path)
        .with_target(false)
        // Build the subscriber
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();


    let db_actor_handle = DbActorHandle::new().await;
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/new", post(handle_new_reading))
        .route("/last", get(fetch_last_reading))
        .layer(Extension(db_actor_handle.clone()));

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Response<String> {
    //let instance: Reading = Reading { value: 4.1_f64 };
    //let result = serde_json::to_string(&instance);
    //res
    //Response::new(StatusCode::ACCEPTED)
    Response::new("Hola".to_string())
}

async fn handle_new_reading(
    Extension(db_actor_handle): Extension<DbActorHandle>,
    new_reading: axum::extract::Json<Reading>,
) {
    // "axum::extract::Json<Reading>" extractS the JSON content present in the body of the request
    // and converts it to an instance of 'Reading'
    info!("New reading received from node {}", new_reading.value);

    //Use the DbActorHandle
    db_actor_handle
        .save_new_reading(Reading {
            value: new_reading.value,
        })
        .await;
}

async fn fetch_last_reading(
    Extension(db_actor_handle): Extension<DbActorHandle>,
) -> Response<String> {
    //Use the DbActorHandle to request DbActor the last reading
    let last_reading = db_actor_handle.get_last_reading().await;
    let last_reading_as_json = serde_json::to_string(&last_reading);
    let response = match last_reading_as_json {
        Ok(last_reading) => Response::new(last_reading),
        Err(_) => Response::new("Error".to_string()),
    };
    response
}

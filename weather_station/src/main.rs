//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::http::{Response, StatusCode};
use axum::{routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

#[derive(Serialize, Deserialize, Debug)]
struct Reading {
    value: f64,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/new", post(handle_new_reading));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Response<String> {
    //let instance: Reading = Reading { value: 4.1_f64 };
    //let result = serde_json::to_string(&instance);
    //res
    //Response::new(StatusCode::ACCEPTED)
    Response::new("hola".to_string())
}

async fn handle_new_reading(new_reading: axum::extract::Json<Reading>) {
    // "axum::extract::Json<Reading>" extractS the JSON content present int the body of the request
    // and converts it into an instance of 'Reading'
    println!("New reading received from node{}", new_reading.value);
    //TODO: send this to the DB actor in charge of saving in sqlite
}

//---------------------DB ACTOR-------------------

// DB actor
struct DbActor {
    inbox: mpsc::Receiver<DbActorMessage>,
}

// Define the type of messages we can send to the actor
enum DbActorMessage {
    SaveNewReading {
        respond_to: oneshot::Sender<bool>,
        new_reading: Reading,
    },
}

impl DbActor {
    fn new(inbox: mpsc::Receiver<DbActorMessage>) -> Self {
        DbActor { inbox: inbox }
    }
    fn handle_db_actor_inbox_msg(&mut self, incoming_msg: DbActorMessage) {
        match incoming_msg {
            DbActorMessage::SaveNewReading {
                respond_to,
                new_reading,
            } => {
                let result: bool = true; //TODO: write to DB and return a boolean if success
                let _ = respond_to.send(result);
            }
        }
    }
}

async fn run_db_actor(mut actor: DbActor) {
    while let Some(msg) = actor.inbox.recv().await {
        actor.handle_db_actor_inbox_msg(msg);
    }
}

// Handle to communicate with the DbActor
#[derive(Clone)]
pub struct DbActorHandle {
    sender: mpsc::Sender<DbActorMessage>,
}

impl DbActorHandle {
    pub fn new() -> Self {
        let (sender_to_db_actor, receiver_at_db_actor) = mpsc::channel(10);
        let db_actor = DbActor::new(receiver_at_db_actor);
        tokio::spawn(run_db_actor(db_actor));
        Self {
            sender: sender_to_db_actor,
        }
    }

    pub fn save_new_reading() {
        println!("Passing new reading to DBActor");
    }
}

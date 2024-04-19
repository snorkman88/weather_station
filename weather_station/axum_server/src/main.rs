//! Run with
//!
//! ```not_rust
//! cargo watch -x run
//! ```

mod db_controller;
use db_controller::Database;

use std::string;

use axum::http::{Response, StatusCode};
use axum::{Error, Extension};
use axum::{routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reading {
    value: f64,
}

#[tokio::main]
async fn main() {
    let db_actor_handle = DbActorHandle::new().await;
    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/new", post(handle_new_reading))
        .route("/last", get(fetch_last_reading))
        .layer(Extension(db_actor_handle.clone()));

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

async fn handle_new_reading(
    Extension(db_actor_handle): Extension<DbActorHandle>,
    new_reading: axum::extract::Json<Reading>,
) {
    // "axum::extract::Json<Reading>" extractS the JSON content present in the body of the request
    // and converts it to an instance of 'Reading'
    println!("New reading received from node {}", new_reading.value);

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

//---------------------DB ACTOR-------------------

struct DbActor {
    inbox: mpsc::Receiver<DbActorMessage>,
    db: Database
}

// Define the type of messages we can send to the actor
#[derive(Debug)]
enum DbActorMessage {
    SaveNewReading {
        respond_to: oneshot::Sender<bool>,
        new_reading: Reading,
    },
    GetLastReading {
        respond_to: oneshot::Sender<Reading>,
    },
}

impl DbActor {
    async fn new(inbox: mpsc::Receiver<DbActorMessage>) -> Self {
        let db = Database::new("mysql://root:root@mariadb:3306").await;
        match db {
            Ok(database_connection) =>{
                println!("Connection to DB was successful.");
                DbActor { inbox: inbox, db:database_connection}
            },
            Err(_) => {
                panic!("No connection to DB. Exiting.")
            }
            
        }
            
    }
    async fn process_db_actor_inbox_msg(&mut self, incoming_msg: DbActorMessage) {
        match incoming_msg {
            DbActorMessage::SaveNewReading {
                respond_to,
                new_reading,
            } => {
                println!("DbActor.process_db_actor_inbox_msg received a new message ");
                // Add logic to interact with SQL database here
                let result: bool = true;
                let _ = respond_to.send(result); //Send the result of the operation back to the handle
            }
            DbActorMessage::GetLastReading { respond_to } => {
                println!("DbActor.get_last_reading received a new message");
                // Add logic to interact with SQL database here
                let last_reading = Reading { value: 37.1_f64 };
                let _ = respond_to.send(last_reading); //Send the result of the operation back to the handle
            }
        }
    }
}

async fn run_db_actor(mut actor: DbActor) {
    while let Some(msg) = actor.inbox.recv().await {
        actor.process_db_actor_inbox_msg(msg).await;
    }
}

// ----------------Handle used to communicate with the DbActor---------------
#[derive(Clone)]
pub struct DbActorHandle {
    sender: mpsc::Sender<DbActorMessage>,
}

impl DbActorHandle {
    pub async fn new() -> Self {
        let (sender_to_db_actor, receiver_at_db_actor) = mpsc::channel(10);
        let db_actor = DbActor::new(receiver_at_db_actor).await;
        tokio::spawn(run_db_actor(db_actor)); //This will run the whole DbActor in the background
        Self {
            sender: sender_to_db_actor,
        }
    }

    pub async fn save_new_reading(self, new_reading: Reading) {
        println!("Passing new reading to DBActor");
        let (send, recv) = oneshot::channel();
        let msg = DbActorMessage::SaveNewReading {
            respond_to: send,
            new_reading: new_reading,
        };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check the
        // failure twice.
        let _ = self.sender.send(msg).await;
        let result = recv.await.expect("Actor task has been killed");
        println!("The DbActor returned {:?} to the DBhandle", result)
    }

    pub async fn get_last_reading(self) -> Reading {
        println!("Requesting last reading to DbActor");
        let (send, recv) = oneshot::channel();
        let msg = DbActorMessage::GetLastReading { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check the
        // failure twice.
        let _ = self.sender.send(msg).await;
        let result = recv.await.expect("Actor task has been killed");
        result
    }
}

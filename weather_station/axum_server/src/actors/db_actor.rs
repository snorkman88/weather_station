use tracing::{debug, error, info, span, Level};
use tokio::sync::{mpsc, oneshot};
use serde::{Deserialize, Serialize};

use super::db_controller::Database;


#[derive(Serialize, Deserialize, Debug)]
pub struct Reading {
    pub value: f64,
}

//---------------------DB ACTOR-------------------

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

struct DbActor {
    inbox: mpsc::Receiver<DbActorMessage>,
    db: Database,
}

impl DbActor {
    async fn new(inbox: mpsc::Receiver<DbActorMessage>) -> Self {

        let db = Database::new("mysql://root:root@mariadb:3306").await;
        match db {
            Ok(database_connection) => {
                info!("Connection to DB was successful.");
                DbActor {
                    inbox: inbox,
                    db: database_connection,
                }
            }
            Err(_) => {
                error!("No connection to DB. Exiting.");
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
                info!("DbActor.process_db_actor_inbox_msg received a new message ");
                // Add logic to interact with SQL database here
                let result: bool = true;
                let _ = respond_to.send(result); //Send the result of the operation back to the handle
            }
            DbActorMessage::GetLastReading { respond_to } => {
                info!("DbActor.get_last_reading received a new message");
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
        debug!("Passing new reading to DBActor");
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
        debug!("The DbActor returned {:?} to the DBhandle", result)
    }

    pub async fn get_last_reading(self) -> Reading {
        debug!("Requesting last reading to DbActor");
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

use std::str::FromStr;

use actix_web::rt::net::{TcpStream, TcpListener};
use log::{info, debug};
use serde_json::Value;

use base64::{Engine as _, engine::general_purpose};

use crate::reader;

use polodb_core::{Database, bson::doc};


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Query {
    sql: String,
    bindings: Vec<String>,
    execution_time: f32
}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionLog {
    uri: String,
    queries: Vec<Query>,
    session_id: String,
    requester_id: String,
    datetime: String
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query: {}", self.sql)
    }
}

impl Query {
    pub fn replace_bindings(&mut self) -> String {
        let mut sql = self.sql.clone();
        self.bindings.iter().enumerate().for_each(|(_i, v)| {
            sql = sql.replacen('?', v.as_str(), 1);
        });
        self.sql = sql.clone();
        sql
    }
}

impl SessionLog {
    pub fn get_queries(&self) -> Vec<Query> {
        self.queries.clone()
    }

    pub fn set_queries(&mut self, new_queries: Vec<Query>) -> Self {
        self.queries = new_queries;
        self.clone()
    }
}

pub async fn connection(db: &Database) {
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();

    info!("Agent listening on port 7878");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        process(socket, db).await;
    }
}

pub async fn process(socket: TcpStream, db: &Database) {
    info!("Connection established!");
    info!("Processing...");

    let received = reader::handle_connection(socket).await.unwrap();
    debug!("Received: {}", received);

    let data: Value = serde_json::from_str(received.as_str()).unwrap();
    debug!("Data: {:?}", data);

    let mut queries: Vec<Query> = vec![];

    let mut uri = String::new();
    let mut requester_id = String::new();

    debug!("{}", serde_json::to_string_pretty(data.as_object().unwrap()).unwrap());

    let datetime = String::from(data["datetime"].as_str().unwrap());
    let mut session_id = String::from("None");

    data["context"].as_object().unwrap().iter().for_each(|(k, v)| {
        match k.as_str() {
            "session_id" => {
                session_id = String::from(v.as_str().unwrap());
                info!("session_id: {}", v);
            },
            "requester_id" => {
                requester_id = String::from(v.as_str().unwrap());
                info!("requester_id: {}", v);
            },
            "uri" => {
                uri = String::from(v.as_str().unwrap());
                info!("uri: {}", v);
            },
            "queries" => {
                debug!("#### queries ####");
                debug!("{}, {}", k, v);
                v.as_array().unwrap().iter().for_each(|v| {
                    let query = v.as_object().unwrap();
                    let decoded_sql = general_purpose::STANDARD
                        .decode(String::from_str(query["sql"].as_str().unwrap()).as_ref().unwrap()).unwrap();
                    let sql = String::from_utf8(decoded_sql).unwrap();

                    let decoded_query = Query {
                        sql,
                        bindings: query["bindings"].as_array().unwrap().iter().map(|v| {
                            // TODO Handle strings and numbers differently
                            // info!("{}", v);
                            // info!("{}", String::from(v.to_string()));
                            return String::from(v.to_string());
                        }).collect(),
                        execution_time: query["execution_time"].as_f64().unwrap_or(0.0) as f32
                    };

                    queries.push(decoded_query.clone());
                });
            },
            _ => {}
        }
    });


    let session_log = SessionLog {
        uri,
        queries,
        session_id,
        requester_id,
        datetime
    };

    let collection = db.collection("session_logs");
    let insert = collection.insert_one(session_log);
    debug!("insert: {:?}", insert);

}


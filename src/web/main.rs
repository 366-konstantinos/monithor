use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, post, delete};
use actix_web_static_files::ResourceFiles;
use log::info;
use actix_web::rt::spawn;
use polodb_core::{Database, bson::doc};
use serde_derive::Deserialize;



use std::sync::Arc;
use crate::agent::{SessionLog, Query as AgentQuery};


mod agent;
// mod connection;
// mod frame;
mod reader;


include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Clone)]
struct AppState {
    db: Arc<Database>
}

#[derive(Deserialize)]
struct Modifier {
    replace_bindings: Option<bool>,
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[get("/api/{name}")]
async fn hello2(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}


#[get("/sessions")]
async fn get_sessions(data: web::Data<AppState>) -> impl Responder {
    let mut v: Vec<SessionLog> = vec![];
    let collection = data.db.collection::<SessionLog>("session_logs").find(None).unwrap();
    for col in collection {
        v.push(col.unwrap());
    }

    info!("get_sessions: {:?}", v);


    HttpResponse::Ok().json(v)
}


#[delete("/sessions")]
async fn clear_sessions(data: web::Data<AppState>) -> impl Responder {
    let collection = data.db.collection::<SessionLog>("session_logs").drop().unwrap();
    info!("clear_sessions: {:?}", collection);
    HttpResponse::Ok().body("OK")
}


#[get("/sessions/{uuid}")]
async fn get_session_by_uuid(data: web::Data<AppState>, path: web::Path<String>, modifier: web::Query<Modifier>) -> impl Responder {
    let session_uuid = path.into_inner();
    let session_log: Option<SessionLog> = data.db.collection::<SessionLog>("session_logs").find_one(doc! {"session_id": session_uuid }).unwrap();

    let session_log_response = if modifier.replace_bindings.unwrap_or(false) {
        let session_log_modified: SessionLog = session_log.clone().unwrap();
        let mut new_queries: Vec<AgentQuery> = vec![];
        let queries: Vec<AgentQuery> = session_log_modified.get_queries();

        for mut query in queries {
            query.replace_bindings();
            new_queries.push(query.clone());
        }
        let new_session_log = session_log.clone().unwrap().set_queries(new_queries.clone());

        info!("session_log_modified: {:?}", new_queries.clone());
        Some(new_session_log)
    } else {
        session_log.clone()
    };

    HttpResponse::Ok().json(session_log_response)
}


#[post("/log")]
async fn log_session() -> impl Responder {
    info!("test post");
    HttpResponse::Ok().body("test")
}

fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { 
                info!("test");
                HttpResponse::Ok().body("test") 
            }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );

    cfg.service(log_session).service(get_sessions).service(get_session_by_uuid).service(clear_sessions);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Arc::new(Database::open_file("monithor.db").unwrap());
    let app_state = web::Data::new(AppState {
        db: db.clone()
    });

    let ip = std::net::IpAddr::from(std::net::Ipv4Addr::from([0, 0, 0, 0]));
    let port = 8899;
    let socket = std::net::SocketAddr::new(ip, port);


    spawn(async move {
        agent::connection(&db).await;
    });
    env_logger::init();
    info!("Starting server on {}", socket);

    HttpServer::new(move || {
        let generated = generate();
        App::new()
            .app_data(web::Data::clone(&app_state))
            .service(web::scope("/api").configure(scoped_config))
            // .route("/{filename:.*}", web::get().to(index))
            // .service(Files::new("/", "./static/root/").index_file("index.html"))
            .service(ResourceFiles::new("/", generated))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind((socket.ip().to_string().as_str(), socket.port()))?
    .run()
    .await
}

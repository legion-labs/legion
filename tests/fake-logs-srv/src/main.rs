//! Legion Fake Losg Server
//!
//! Temporary fake server that serves logs through http requests and web sockets.

#![allow(clippy::useless_let_if_seq)]

use std::{fmt::Display, fs::File, path::Path, sync::Arc};

use chrono::{DateTime, Utc};
use clap::{ArgEnum, Parser};
use futures::{stream::StreamExt, SinkExt};
use lipsum::{lipsum, lipsum_words};
use poem::{
    get, handler,
    listener::TcpListener,
    middleware::{AddData, Cors},
    web::{
        websocket::{Message, WebSocket},
        Data,
    },
    EndpointExt, IntoResponse, Route, Server,
};
use poem_openapi::{param::Query, payload::Json, Object, OpenApi, OpenApiService};
use rand::seq::SliceRandom;
use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};

const LOG_LEVELS: [&str; 5] = ["debug", "error", "info", "trace", "warn"];

const MAX_SIZE: u32 = 1_000;

#[derive(Debug, Clone, ArgEnum)]
enum DbType {
    File,
    InMemory,
}

impl Default for DbType {
    fn default() -> Self {
        Self::InMemory
    }
}

#[derive(Debug, Default, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 't', long, default_value = "10000")]
    total_count: u32,
    #[clap(arg_enum, short = 'd', long, default_value = "in-memory")]
    db_type: DbType,
}

struct Api;

#[derive(Debug, Clone, Object, Serialize, FromRow)]
struct Log {
    id: u32,
    message: String,
    severity: String,
    target: String,
    datetime: DateTime<Utc>,
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Clone, Object, Serialize)]
struct InsertLog {
    message: String,
    severity: String,
    target: String,
    datetime: DateTime<Utc>,
}

impl InsertLog {
    fn random() -> Self {
        Self {
            message: lipsum(30),
            severity: (*LOG_LEVELS.choose(&mut rand::thread_rng()).unwrap()).to_string(),
            target: lipsum_words(1),
            datetime: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
struct LogMessage {
    log: Log,
    total_count: u32,
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl LogMessage {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

#[derive(Debug, Default, Object)]
struct Pagination {
    prev: Option<String>,
    next: Option<String>,
    total_count: u32,
}

#[derive(Debug, Object)]
struct LogsResponse {
    data: Vec<Log>,
    pagination: Pagination,
}

#[derive(FromRow)]
struct TotalCount {
    total_count: u32,
}

async fn get_total_count(pool: &SqlitePool) -> Result<u32, Box<dyn std::error::Error>> {
    let TotalCount { total_count } =
        sqlx::query_as::<_, TotalCount>("select count(*) as total_count from logs")
            .fetch_one(pool)
            .await?;

    Ok(total_count)
}

async fn insert_log(pool: &SqlitePool) -> Result<Log, Box<dyn std::error::Error>> {
    let log = InsertLog::random();

    let log =    sqlx::query_as::<_, Log>(
        "insert into logs (message, severity, target, datetime) values (?1, ?2, ?3, ?4) returning *",
    )
    .bind(log.message)
    .bind(log.severity)
    .bind(log.target)
    .bind(log.datetime)
    .fetch_one(pool)
    .await?;

    Ok(log)
}

#[OpenApi]
impl Api {
    #[oai(path = "/logs", method = "get")]
    async fn logs(
        &self,
        pool: Data<&SqlitePool>,
        size: Query<u32>,
        before: Query<Option<u32>>,
        after: Query<Option<u32>>,
    ) -> Json<LogsResponse> {
        let total_count = get_total_count(pool.0).await.unwrap();

        let size = if size.0 > MAX_SIZE { MAX_SIZE } else { size.0 };

        let (offset, limit) = before
            .0
            .map(|before| {
                if before >= size {
                    (before - size, size)
                } else {
                    (0, before)
                }
            })
            .or_else(|| {
                after.0.map(|after| {
                    if after >= total_count {
                        (total_count, 0)
                    } else if size > total_count - after {
                        (after, total_count - after)
                    } else {
                        (after, size)
                    }
                })
            })
            .unwrap_or((0, size));

        let logs =
            sqlx::query_as::<_, Log>("select * from logs order by id desc limit ?1 offset ?2")
                .bind(limit)
                .bind(offset)
                .fetch_all(pool.0)
                .await
                .unwrap();

        let prev = (offset != 0).then(|| format!("/api/logs?size={}&before={}", limit, offset));

        let next = (offset + limit < total_count)
            .then(|| format!("/api/logs?limit={}&after={}", limit, offset + limit));

        Json(LogsResponse {
            data: logs,
            pagination: Pagination {
                prev,
                next,
                total_count,
            },
        })
    }
}

#[handler]
#[allow(clippy::needless_pass_by_value)]
async fn ws(ws: WebSocket, pool: Data<&SqlitePool>) -> impl IntoResponse {
    let mut interval = Arc::new(Mutex::new(time::interval(Duration::from_millis(2_000))));

    let pool = pool.0.clone();

    ws.on_upgrade(move |socket| async move {
        let interval = Arc::clone(&interval);

        let pool = pool.clone();

        let (mut sink, _stream) = socket.split();

        tokio::spawn(async move {
            let interval = Arc::clone(&interval);

            let pool = pool.clone();

            loop {
                interval.lock().await.tick().await;

                let log = insert_log(&pool).await.unwrap();

                let total_count = get_total_count(&pool).await.unwrap();

                let message = LogMessage { log, total_count };

                if let Err(error) = sink.send(Message::Binary(message.to_bytes())).await {
                    eprintln!("Message: {:?}, error: {}", message, error);

                    break;
                }
            }
        });
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let db_uri = match args.db_type {
        DbType::InMemory => "sqlite::memory:".into(),
        DbType::File => {
            let db_path = Path::new(file!())
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("logs.db");

            if !db_path.exists() {
                File::create(&db_path)?;
            }

            format!("sqlite://{}", db_path.to_string_lossy())
        }
    };

    let pool = SqlitePoolOptions::new().connect(&db_uri).await.unwrap();

    let logs_table =
        sqlx::query("select name from sqlite_schema where type = 'table' and name = 'logs'")
            .fetch_one(&pool)
            .await;

    if let Err(sqlx::Error::RowNotFound) = logs_table {
        println!(
            "Building db {} with {} rows, it might take some time",
            db_uri, args.total_count
        );

        sqlx::query(
            "create table logs (
                id integer primary key autoincrement,
                message text,
                severity text,
                target text,
                datetime text
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        for index in 0..args.total_count {
            if index != 0 && index % 1_000 == 0 {
                println!("Built index: {}", index);
            }

            insert_log(&pool).await.unwrap();
        }
    }

    let api_service =
        OpenApiService::new(Api, "fake-logs", "1.0").server("http://localhost:4000/api");

    let ui = api_service.swagger_ui();

    println!("Starting server on port 4000");

    Server::new(TcpListener::bind("127.0.0.1:4000"))
        .run(
            Route::new()
                .at("/ws", get(ws))
                .nest("/api", api_service)
                .nest("/", ui)
                .with(Cors::new())
                .with(AddData::new(pool)),
        )
        .await
}

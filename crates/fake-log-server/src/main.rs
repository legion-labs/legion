#![allow(clippy::useless_let_if_seq)]

use std::{borrow::Cow, fmt::Display, sync::Arc};

use chrono::{DateTime, Utc};
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
use poem_openapi::{
    param::Query,
    payload::Json,
    registry::{self, MetaSchema, MetaSchemaRef},
    types::{ParseError, ParseFromJSON, ParseResult, ToJSON, Type},
    Object, OpenApi, OpenApiService,
};
use rand::seq::SliceRandom;
use serde::{Serialize, Serializer};
use tokio::{
    sync::Mutex,
    time::{self, Duration},
};

const LOG_LEVELS: [log::Level; 5] = [
    log::Level::Debug,
    log::Level::Error,
    log::Level::Info,
    log::Level::Trace,
    log::Level::Warn,
];

const INIT_TOTAL_COUNT: u32 = 500_000;

const MAX_SIZE: u32 = 10_000;

struct Api;

#[derive(Debug, Clone)]
struct LogLevel(log::Level);

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = match self.0 {
            log::Level::Debug => "debug",
            log::Level::Error => "error",
            log::Level::Info => "info",
            log::Level::Trace => "trace",
            log::Level::Warn => "warn",
        };

        serializer.serialize_str(string)
    }
}

impl Type for LogLevel {
    const IS_REQUIRED: bool = true;

    type RawValueType = Self;

    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        "string(log-level)".into()
    }

    fn schema_ref() -> registry::MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format("string", "log-level")))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

impl ToJSON for LogLevel {
    fn to_json(&self) -> Option<serde_json::Value> {
        let string = match self.0 {
            log::Level::Debug => "debug",
            log::Level::Error => "error",
            log::Level::Info => "info",
            log::Level::Trace => "trace",
            log::Level::Warn => "warn",
        };

        Some(serde_json::Value::String(string.into()))
    }
}

impl ParseFromJSON for LogLevel {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        if let Some(value) = value {
            if let Some(value) = value.as_str() {
                let value = match value {
                    "debug" => log::Level::Debug,
                    "error" => log::Level::Error,
                    "info" => log::Level::Info,
                    "trace" => log::Level::Trace,
                    "warn" => log::Level::Warn,
                    unknown_value => {
                        return Err(ParseError::custom(format!(
                            "Unknown value {}",
                            unknown_value
                        )))
                    }
                };

                Ok(Self(value))
            } else {
                Err(ParseError::custom("Provided value is not a string"))
            }
        } else {
            Err(ParseError::custom("No value provided"))
        }
    }
}

#[derive(Debug, Clone, Object, Serialize)]
struct Log {
    id: u32,
    message: String,
    severity: LogLevel,
    target: String,
    timestamp: DateTime<Utc>,
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Log {
    fn random(id: u32) -> Self {
        Self {
            id,
            message: lipsum(30),
            severity: LogLevel(*LOG_LEVELS.choose(&mut rand::thread_rng()).unwrap()),
            target: lipsum_words(1),
            timestamp: Utc::now(),
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

#[OpenApi]
impl Api {
    #[oai(path = "/logs", method = "get")]
    async fn logs(
        &self,
        total_count: Data<&Arc<Mutex<u32>>>,
        size: Query<u32>,
        before: Query<Option<u32>>,
        after: Query<Option<u32>>,
    ) -> Json<LogsResponse> {
        let total_count = *total_count.0.lock().await;

        let size = if size.0 > MAX_SIZE { MAX_SIZE } else { size.0 };

        let (first, size) = before
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

        let logs = (0..size)
            .map(|index| Log::random(total_count - first - size + index + 1))
            .collect();

        let prev = (first != 0).then(|| format!("/api/logs?size={}&before={}", size, first));

        let next = (first + size < total_count)
            .then(|| format!("/api/logs?size={}&after={}", size, first + size));

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
fn ws(ws: WebSocket, total_count: Data<&Arc<Mutex<u32>>>) -> impl IntoResponse {
    let mut interval = Arc::new(Mutex::new(time::interval(Duration::from_millis(2_000))));
    let total_count = Arc::clone(total_count.0);

    ws.on_upgrade(move |socket| async move {
        let interval = Arc::clone(&interval);
        let total_count = Arc::clone(&total_count);
        let (mut sink, _stream) = socket.split();

        tokio::spawn(async move {
            let interval = Arc::clone(&interval);
            let total_count = Arc::clone(&total_count);

            loop {
                interval.lock().await.tick().await;

                let mut total_count = total_count.lock().await;

                *total_count += 1;

                if sink
                    .send(Message::Binary(
                        LogMessage {
                            log: Log::random(*total_count),
                            total_count: *total_count,
                        }
                        .to_bytes(),
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug,fake-log=debug");
    }

    let total_count = Arc::new(Mutex::new(INIT_TOTAL_COUNT));

    tracing_subscriber::fmt::init();

    let api_service =
        OpenApiService::new(Api, "fake-log", "1.0").server("http://localhost:4000/api");

    let ui = api_service.swagger_ui();

    Server::new(TcpListener::bind("127.0.0.1:4000"))
        .run(
            Route::new()
                .at("/ws", get(ws))
                .nest("/api", api_service)
                .nest("/", ui)
                .with(Cors::new())
                .with(AddData::new(total_count)),
        )
        .await
}

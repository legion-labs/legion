pub mod client;
pub mod multiplexer_service;
pub mod server;

mod buf;
mod consts;
mod errors;
pub(crate) mod web;

pub use client::{GrpcClient, GrpcWebClient};
pub use errors::{Error, Result};
pub use multiplexer_service::MultiplexerService;
pub use server::{HybridServer, RestServer, Server};

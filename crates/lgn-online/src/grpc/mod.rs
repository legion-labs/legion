pub mod client;
pub mod multiplexer_service;
pub mod server;

mod authentication;
mod aws_lambda_handler;
mod buf;
mod consts;
mod errors;
mod web;

pub use authentication::AuthenticatedClient;
pub use client::{GrpcClient, GrpcWebClient};
pub use errors::{Error, Result};
pub use multiplexer_service::MultiplexerService;
pub use server::Server;

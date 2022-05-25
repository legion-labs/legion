pub mod api {
    #![allow(unused_imports)]
    #![allow(unused_mut)]
    #![allow(dead_code)]

    lgn_online::include_apis!(governance, session);
}

mod errors;
mod server;

pub use errors::{Error, Result};
pub use server::Server;

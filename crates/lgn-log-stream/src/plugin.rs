use std::sync::Arc;

use lgn_app::prelude::{App, Plugin};
use lgn_grpc::SharedRouter;

use crate::server::{Server, TraceEventsReceiver};

#[derive(Default)]
pub struct LogStreamPlugin;

impl Plugin for LogStreamPlugin {
    fn build(&self, app: &mut App) {
        let receiver = app.world.resource_mut::<TraceEventsReceiver>();

        let server = Arc::new(Server::new(receiver.clone()));

        let router = app.world.resource_mut::<SharedRouter>().into_inner();

        router.register_routes(crate::api::log::server::register_routes, server);
    }
}

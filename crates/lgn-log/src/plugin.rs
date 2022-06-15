use std::sync::Arc;

use lgn_app::prelude::{App, Plugin};
use lgn_async::receiver::SharedUnboundedReceiver;
use lgn_grpc::SharedRouter;
use tokio::sync::broadcast::error::RecvError;

use crate::{
    api::log::TraceEvent,
    server::{Server, TraceEventDeque},
};

pub type TraceEventsReceiver = SharedUnboundedReceiver<crate::TraceEvent>;

#[derive(Default)]
pub struct LogStreamPlugin;

impl Plugin for LogStreamPlugin {
    fn build(&self, app: &mut App) {
        let mut trace_events = TraceEventDeque::new();

        let server = Arc::new(Server::new(trace_events.clone()));

        {
            let router = app.world.resource_mut::<SharedRouter>().into_inner();

            router.register_routes(crate::api::log::server::register_routes, server);
        }

        let trace_events_receiver = app.world.resource_mut::<TraceEventsReceiver>().clone();

        let rt = app.world.resource::<lgn_async::TokioAsyncRuntime>();

        rt.start_detached(async move {
            loop {
                match trace_events_receiver.lock().await.recv().await {
                    Ok(crate::TraceEvent::Message {
                        target,
                        message,
                        level,
                        time,
                    }) => {
                        trace_events.push_back(TraceEvent {
                            // Value for Level starts at 1 so we simply decrement the level to get the proper value at runtime
                            level: (level as i32 - 1).try_into().unwrap(),
                            message,
                            target,
                            time,
                        });
                    }
                    Ok(_trace_event) => {
                        // Ignoring other events for now
                    }
                    Err(RecvError::Lagged(_)) => {
                        // Ingoring lags
                    }
                    Err(RecvError::Closed) => return,
                }
            }
        });
    }
}

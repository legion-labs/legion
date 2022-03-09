use super::common_types::{NodeData, NodeInitial, NodeType, RemoteExecutionArgs, ServerData};
use crate::node_crunch::{
    nc_config::NCConfiguration, nc_error::NCError, nc_node::NCNode, nc_node::NCNodeStarter,
    nc_node::NCNodeStatus,
};
use lgn_tracing::info;

struct Worker {}

impl NCNode for Worker {
    type NodeTypeT = NodeInitial;
    type InitialDataT = ();
    type NewDataT = ServerData;
    type ProcessedDataT = NodeData;
    type CustomMessageT = ();

    // This says whether we're in "worker" mode or in "lib client" mode.
    fn get_node_type(&mut self) -> Result<Self::NodeTypeT, NCError> {
        info!("Connected to server...");
        Ok(NodeInitial {
            node_type: NodeType::Worker,
        })
    }

    fn process_data_from_server(
        &mut self,
        data: &Self::NewDataT,
    ) -> Result<NCNodeStatus<Self::ProcessedDataT>, NCError> {
        info!("Received & executing workload...");
        let output_archive = crate::compiler_node::remote_data_executor::execute_sandbox_compiler(
            &data.input_archive,
        )?;
        let result = NodeData {
            request_id: data.request_id,
            output_archive,
        };
        info!("Workload executed...");
        Ok(NCNodeStatus::Ready(result))
    }
}

fn config(options: RemoteExecutionArgs) -> NCNodeStarter {
    let configuration = NCConfiguration {
        url: options.url,
        compress: false,
        encrypt: true,
        delay_request_data: 1, // sec
        // The key should be read from a config file
        key: "ZKS1GQ3MYWEKFILSN6KESXU2GD9015CH".to_string(),
        ..NCConfiguration::default()
    };
    NCNodeStarter::new(configuration)
}

/// Starts a worker node with the given configuration.
pub fn run_worker(options: RemoteExecutionArgs) {
    let mut node = Worker {};
    config(options).start(&mut node).unwrap();
}

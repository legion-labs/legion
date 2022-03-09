use crate::node_crunch::{
    nc_config::NCConfiguration, nc_error::NCError, nc_node::NCNode, nc_node::NCNodeStarter,
    nc_node::NCNodeStatus,
};
use lgn_tracing::info;

use super::common_types::{NodeData, NodeInitial, NodeType, RemoteExecutionArgs, ServerData};

struct RemoteExecutionNode {
    input_archive: Vec<u8>,
    output_archive: Vec<u8>,
}

impl NCNode for RemoteExecutionNode {
    type NodeTypeT = NodeInitial;
    type InitialDataT = ();
    type NewDataT = ServerData;
    type ProcessedDataT = NodeData;
    type CustomMessageT = ();

    // This says whether we're in "worker" mode or in "lib client" mode.
    // For the "worker" case, we will be ticked through the process_data_from_server below.
    // For the "lib client", we simply send the archive for execution to the server, which will re-dispatch it to a worker.
    fn get_node_type(&mut self) -> Result<Self::NodeTypeT, NCError> {
        info!("Connected to server...");
        Ok(NodeInitial {
            node_type: NodeType::InitiatingClient(self.input_archive.clone()),
        })
    }

    fn process_data_from_server(
        &mut self,
        data: &Self::NewDataT,
    ) -> Result<NCNodeStatus<Self::ProcessedDataT>, NCError> {
        info!("Received data from server...");
        self.output_archive = data.input_archive.clone();
        // Since this is a call from the client app, we're done as soon as the service sends up the resulting data.
        Ok(NCNodeStatus::Exiting)
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

/// Send the workload from a client.
pub fn send_receive_workload(server_addr: &str, input_archive: Vec<u8>) -> Vec<u8> {
    info!("Sending workload...");
    let mut node = RemoteExecutionNode {
        input_archive,
        output_archive: vec![],
    };
    let options = RemoteExecutionArgs {
        server: false,
        url: server_addr.to_owned(),
    };
    config(options).start(&mut node).unwrap();
    info!("Received workload...");
    node.output_archive.clone()
}

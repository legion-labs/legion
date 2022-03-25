use clap::Parser;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct RemoteExecutionArgs {
    #[clap(short = 's', long = "server")]
    pub server: bool,

    #[clap(long = "url", default_value = "lgn://127.0.0.1:2022")]
    pub url: String,
}

/// Represents a unit of workload for a "worker" client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerData {
    /// Arbitrary number that represents a request to a worker.
    pub request_id: u64,

    /// A message which contains the request job to be the executed on the remote worker.
    pub input_msg: String,
}

/// The computed workload returned by the "worker" client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    /// The request number; must match the request sent in `ServerData`.
    pub request_id: u64,

    /// A message that contains the output of the compilation process which was executed on the remote worker.
    pub output_msg: String,
}

/// Tells the kind of the remote client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// A remote worker that accepts jobs from the service.
    Worker,

    /// A connecting client that sends jobs to the service.
    InitiatingClient(String),
}

/// A structure that is returned by a client on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInitial {
    /// Describes what kind of client the server is connecting to.
    pub node_type: NodeType,
}

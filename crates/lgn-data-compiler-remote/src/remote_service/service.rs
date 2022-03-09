use std::collections::{HashMap, VecDeque};

use super::common_types::{NodeData, NodeInitial, NodeType, RemoteExecutionArgs, ServerData};
use crate::node_crunch::{
    nc_config::NCConfiguration, nc_error::NCError, nc_node_info::NodeID, nc_server::NCJobStatus,
    nc_server::NCServer, nc_server::NCServerStarter,
};
use freelist::{FreeList, Idx};
use lgn_tracing::{debug, error};

#[derive(Debug, Clone)]
struct Job {
    initiating_node_id: NodeID,
    input_archive: Vec<u8>,
    output_archive: Vec<u8>,
}

#[derive(Debug, Clone)]
struct RemoteWork {
    job_id: Idx,
    request_id: u64,
}

#[derive(Debug, Clone)]
struct ServerState {
    jobs: FreeList<Job>,
    // Queued, but not yet dispatched, jobs.
    queue: VecDeque<Idx>,
    // Jobs that are currently being processed by remote workers.
    workers_in_progress: HashMap<NodeID, RemoteWork>,
    // Initiating clients.
    clients_waiting: HashMap<NodeID, Idx>,
    // Counter
    request_id_counter: u64,
}

impl ServerState {
    fn process_for_client(&self, node_id: NodeID) -> NCJobStatus<ServerData> {
        //info!("process_for_client...");
        let job_id = self.clients_waiting[&node_id];
        #[allow(unsafe_code)]
        let job = unsafe { self.jobs.get_unchecked(job_id) };
        if job.output_archive.is_empty() {
            NCJobStatus::Waiting // Still waiting for a worker to finish.
        } else {
            NCJobStatus::Unfinished(ServerData {
                request_id: 0,
                input_archive: job.output_archive.clone(),
            })
        }
    }
    fn process_for_worker(&mut self, node_id: NodeID) -> NCJobStatus<ServerData> {
        //info!("process_for_worker...");
        if let Some(job_id) = self.queue.pop_front() {
            // First ensure that we didn't already dispatch anything to this node.
            assert!(!self.workers_in_progress.contains_key(&node_id));

            let node_data = ServerData {
                request_id: self.request_id_counter,
                #[allow(unsafe_code)]
                input_archive: unsafe { self.jobs.get_unchecked(job_id) }
                    .input_archive
                    .clone(),
            };
            self.workers_in_progress.insert(
                node_id,
                RemoteWork {
                    job_id,
                    request_id: self.request_id_counter,
                },
            );
            self.request_id_counter += 1;

            debug!("preparing job {} for node {}", job_id, node_id);
            NCJobStatus::Unfinished(node_data)
        } else {
            NCJobStatus::Waiting
        }
    }
}

impl NCServer for ServerState {
    type NodeTypeT = NodeInitial;
    type InitialDataT = ();
    type NewDataT = ServerData;
    type ProcessedDataT = NodeData;
    type CustomMessageT = ();

    fn initial_data(
        &mut self,
        node_id: NodeID,
        register_data: &Self::NodeTypeT,
    ) -> Result<Option<Self::InitialDataT>, NCError> {
        match &register_data.node_type {
            NodeType::Worker => Ok(None),
            NodeType::InitiatingClient(archive) => {
                let job_id = self.jobs.add(Job {
                    initiating_node_id: node_id,
                    input_archive: archive.clone(),
                    output_archive: vec![],
                });
                self.clients_waiting.insert(node_id, job_id);
                self.queue.push_back(job_id);
                Ok(None)
            }
        }
    }

    // This is where we provide data to the node.
    fn prepare_data_for_node(
        &mut self,
        node_id: NodeID,
    ) -> Result<NCJobStatus<Self::NewDataT>, NCError> {
        debug!("Server::prepare_data_for_node, node_id: {}", node_id);

        Ok(if self.clients_waiting.contains_key(&node_id) {
            self.process_for_client(node_id)
        } else {
            self.process_for_worker(node_id)
        })
    }

    // This is the result that the node returns.
    fn process_data_from_node(
        &mut self,
        node_id: NodeID,
        node_data: &Self::ProcessedDataT,
    ) -> Result<(), NCError> {
        debug!("Server::process_data_from_node, node_id: {}", node_id);

        let request_id = node_data.request_id;

        if let Some(remote_work) = self.workers_in_progress.get(&node_id) {
            if remote_work.request_id != request_id {
                error!("Requests mismatch!");
                return Err(NCError::NodeMsgMismatch);
            }
            #[allow(unsafe_code)]
            let job = unsafe { self.jobs.get_unchecked_mut(remote_work.job_id) };
            if job.initiating_node_id.is_empty() {
                // The job was abandoned, since the client disconnected.
                self.jobs.remove(remote_work.job_id);
            } else {
                job.output_archive = node_data.output_archive.clone();
            }

            self.workers_in_progress.remove(&node_id);
            Ok(())
        } else {
            error!(
                "Mismatch data, should be processing with node_id: {}",
                node_id
            );
            Err(NCError::NodeIDMismatch(node_id))
        }
    }

    // Update our pending queue if some nodes have crashed or lost the network connection.
    fn heartbeat_timeout(&mut self, nodes: Vec<NodeID>) {
        for n in nodes {
            if let Some(remote_work) = self.workers_in_progress.remove(&n) {
                // Put it back in the to-do queue.
                self.queue.push_back(remote_work.job_id);
            }
            if let Some(job_id) = self.clients_waiting.remove(&n) {
                // If the client has crashed, let the job complete.
                #[allow(unsafe_code)]
                unsafe { self.jobs.get_unchecked_mut(job_id) }
                    .initiating_node_id
                    .set_empty();
            }
        }
    }

    fn finish_job(&mut self) {
        //todo!()
    }
}

/// Starts the server with the given configuration
pub fn run_server(options: &RemoteExecutionArgs) {
    let configuration = NCConfiguration {
        url: options.url.clone(),
        compress: false,
        encrypt: true,
        // The key should be read from a config file
        key: "ZKS1GQ3MYWEKFILSN6KESXU2GD9015CH".to_string(),
        ..NCConfiguration::default()
    };

    let server = ServerState {
        jobs: FreeList::new(),
        queue: VecDeque::new(),
        workers_in_progress: HashMap::new(),
        clients_waiting: HashMap::new(),
        request_id_counter: 0,
    };

    NCServerStarter::new(configuration).start(server);
}

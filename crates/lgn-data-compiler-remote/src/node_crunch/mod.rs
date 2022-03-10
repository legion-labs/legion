//! `NodeCrunch` is a crate that allows to distribute computations among multiple nodes.
//! The user of this crate has to implement one trait for the server and one trait for the nodes.
//! Usually the code for the server and the node is inside the same binary and the choice if to
//! run in server mode or node mode is done via configuration or command line argument.
//! See some of the programs in the example folders.

// TODO:
// Examples to add:
// - Add an example with
//   - https://github.com/wahn/rs_pbrt
//   - https://github.com/TwinkleBear/tray_rust
//   - https://github.com/alesgenova/ray-tracer
// - Distributed machine learning ?
// - Distributed black hole simulation ?
// - Distributed crypto mining ?
//
// Own projects that will use node_crunch:
// - Darwin: https://github.com/willi-kappler/darwin-rs
// - GRONN: https://github.com/willi-kappler/gronn

//pub(crate) mod array2d;
pub(crate) mod nc_communicator;
pub(crate) mod nc_config;
pub(crate) mod nc_error;
pub(crate) mod nc_node;
pub(crate) mod nc_node_info;
pub(crate) mod nc_server;

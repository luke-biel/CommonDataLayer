use edge_registry::{EdgeRegistryImpl, RegistryConfig};
use rpc::edge_registry::edge_registry_server::EdgeRegistryServer;
use structopt::StructOpt;
use tonic::transport::Server;

#[tokio::main]
async fn main() {
    let registry = EdgeRegistryImpl::new(RegistryConfig::from_args())
        .await
        .unwrap();
    Server::builder()
        .add_service(EdgeRegistryServer::new(registry))
        .serve(([0, 0, 0, 0], 50110).into())
        .await
        .unwrap();
}

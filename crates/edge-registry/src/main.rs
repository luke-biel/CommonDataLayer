use edge_registry::{EdgeRegistryImpl, RegistryConfig};
use rpc::edge_registry::edge_registry_server::EdgeRegistryServer;
use structopt::StructOpt;
use tonic::transport::Server;

#[tokio::main]
async fn main() {
    let config = RegistryConfig::from_args();
    let registry = EdgeRegistryImpl::new(&config)
        .await
        .unwrap();
    Server::builder()
        .add_service(EdgeRegistryServer::new(registry))
        .serve(([0, 0, 0, 0], config.communication_port).into())
        .await
        .unwrap();
}

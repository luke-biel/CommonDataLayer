use anyhow::Context;
use schema_registry::error::RegistryError;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tonic::transport::Server;
use utils::{metrics, status_endpoints};

#[derive(Deserialize)]
struct Config {
    pub export_dir: Option<PathBuf>,
    pub import_file: Option<PathBuf>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // env_logger::init();
    // let config = envy::from_env::<Config>().context("Env vars not set correctly")?;

    // status_endpoints::serve();
    // metrics::serve();

    // let data_store = SledDatastore::new(&config.db_name).map_err(RegistryError::ConnectionError)?;
    // let registry = SchemaRegistryImpl::new(
    //     data_store,
    //     config.replication_role,
    //     replication_config,
    //     config.pod_name,
    // )
    // .await?;

    // if let Some(export_dir_path) = config.export_dir {
    //     let exported = registry.export_all()?;
    //     let exported = serde_json::to_string(&exported)?;
    //     let export_path = export_path(export_dir_path);
    //     let mut file = File::create(export_path)?;
    //     write!(file, "{}", exported)?;
    // }

    // if let Some(import_path) = config.import_file {
    //     let imported = File::open(import_path)?;
    //     let imported = serde_json::from_reader(imported)?;
    //     registry.import_all(imported)?;
    // }
    //
    Ok(())
}

fn export_path(export_dir_path: PathBuf) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Invalid system time");
    export_dir_path.join(format!("export_{:?}.json", timestamp.as_secs()))
}

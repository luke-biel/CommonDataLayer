use anyhow::Context;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub use metrics;

const METRICS_PORT: u16 = 51805;

pub fn setup_metrics() -> anyhow::Result<()> {
    PrometheusBuilder::new()
        .listen_address(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            METRICS_PORT,
        ))
        .install()
        .context("failed to install Prometheus recorder")
}

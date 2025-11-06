use clap::Parser;
use eventsource_client::{Client as _, ClientBuilder, SSE};
use futures_util::stream::StreamExt;
use graphyne::{GraphiteClient, GraphiteMessage};
use serde::Deserialize;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address of the ESPHome device's event stream
    #[arg(short, long, default_value = "http://esphome-test.fritz.box/events")]
    eventsource_url: String,

    /// Address of the Graphite server
    #[arg(short, long, default_value = "localhost:2003")]
    graphite_server: String,

    /// Prefix for the metric names
    #[arg(short, long, default_value = "home.homecontrol.test")]
    prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StateEvent {
    id: String,
    name: Option<String>,
    icon: Option<String>,
    entity_category: Option<u8>,
    value: Option<f64>,
    state: String,
    uom: Option<String>,
}

fn resolve_address(address: &str) -> Option<SocketAddr> {
    address.to_socket_addrs().ok()?.find(|addr| addr.is_ipv4())
}

fn create_graphite_client(address: &str) -> Result<GraphiteClient, String> {
    let resolved_address = match resolve_address(address) {
        Some(addr) => addr,
        None => {
            return Err(format!(
                "failed to resolve graphite server address: {}",
                address
            ))
        }
    };

    GraphiteClient::builder()
        .address(resolved_address.ip().to_string())
        .port(resolved_address.port())
        .build()
        .map_err(|e| e.to_string())
}

fn rewrite_sensor_name(name: &str) -> String {
    if name.starts_with("temperature_") {
        return name.replace("_", ".");
    }
    if name == "water_level" {
        return "water_level.water_level".to_string();
    }
    name.replace("-", ".").replace("_", ".")
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    env_logger::init();

    let mut graphite_client = match create_graphite_client(&args.graphite_server) {
        Ok(client) => client,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };

    let client = ClientBuilder::for_url(&args.eventsource_url)
        .unwrap()
        .read_timeout(Duration::from_secs(60))
        .build();

    let mut stream = client.stream();

    while let Some(event) = stream.next().await {
        if let Ok(SSE::Event(event)) = event {
            if event.event_type == "state" {
                if let Ok(state_event) = serde_json::from_str::<StateEvent>(&event.data) {
                    if let Some(value) = state_event.value {
                        let sensor_name = state_event.id.strip_prefix("sensor-").unwrap();
                        let metric_name = rewrite_sensor_name(sensor_name);

                        let final_metric_name = if let Some(prefix) = &args.prefix {
                            format!("{}.{}", prefix, metric_name)
                        } else {
                            metric_name
                        };

                        let message = GraphiteMessage::new(&final_metric_name, &value.to_string());
                        if let Err(e) = graphite_client.send_message(&message) {
                            log::error!("failed to send metric to graphite: {}", e);
                        }
                    }
                }
            }
        }
    }
}

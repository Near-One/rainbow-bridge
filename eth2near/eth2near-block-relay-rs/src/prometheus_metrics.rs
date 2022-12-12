use prometheus::{IntCounter, Registry};

use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use warp::Filter;
use warp::Rejection;
use warp::Reply;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref LAST_ETH_SLOT: IntCounter =
        IntCounter::new("last_eth_slot", "Last Ethereum Slot").expect("metric can't be created");
    pub static ref LAST_ETH_SLOT_ON_NEAR: IntCounter =
        IntCounter::new("last_eth_slot_on_near", "Last Ethereum Slot on NEAR")
            .expect("metric can't be created");
    pub static ref LAST_FINALIZED_ETH_SLOT: IntCounter =
        IntCounter::new("last_finalized_eth_slot", "Last Finalized Ethereum Slot")
            .expect("metric can't be created");
    pub static ref LAST_FINALIZED_ETH_SLOT_ON_NEAR: IntCounter = IntCounter::new(
        "last_finalized_eth_slot_on_near",
        "Last Finalized Ethereum Slot on NEAR"
    )
    .expect("metric can't be created");
    pub static ref FAILS_ON_HEADERS_SUBMISSION: IntCounter = IntCounter::new(
        "fails_on_headers_submission",
        "Fails number on Headers Submission"
    )
    .expect("metric can't be created");
    pub static ref FAILS_ON_UPDATES_SUBMISSION: IntCounter = IntCounter::new(
        "fails_on_updates_submission",
        "Fails number on Light Client Updates Submission"
    )
    .expect("metric can't be created");
    pub static ref CHAIN_EXECUTION_BLOCK_HEIGHT_ON_ETH: IntCounter = IntCounter::new(
        "chain_execution_block_height_on_eth",
        "Chain execution block height on eth"
    )
    .expect("metric can't be created");
    pub static ref CHAIN_FINALIZED_EXECUTION_BLOCK_HEIGHT_ON_ETH: IntCounter = IntCounter::new(
        "chain_finalized_execution_block_height_on_eth",
        "Chain finalized execution block height on eth"
    )
    .expect("metric cann't be created");
    pub static ref CHAIN_EXECUTION_BLOCK_HEIGHT_ON_NEAR: IntCounter = IntCounter::new(
        "chain_execution_block_height_on_near",
        "Chain execution block height on near"
    )
    .expect("metric can't be created");
    pub static ref CHAIN_FINALIZED_EXECUTION_BLOCK_HEIGHT_ON_NEAR: IntCounter = IntCounter::new(
        "chain_finalized_execution_block_height_on_near",
        "Chain finalized execution block height on near"
    )
    .expect("metric can't be created");
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(LAST_ETH_SLOT.clone()))
        .expect("last_eth_slot can't be registered");

    REGISTRY
        .register(Box::new(LAST_ETH_SLOT_ON_NEAR.clone()))
        .expect("last_eth_slot_on_near can't be registered");

    REGISTRY
        .register(Box::new(LAST_FINALIZED_ETH_SLOT.clone()))
        .expect("last_finalized_eth_slot can't be registered");

    REGISTRY
        .register(Box::new(LAST_FINALIZED_ETH_SLOT_ON_NEAR.clone()))
        .expect("last_finalized_eth_slot_on_near can't be registered");

    REGISTRY
        .register(Box::new(FAILS_ON_HEADERS_SUBMISSION.clone()))
        .expect("fails_on_header_submission can't be registered");

    REGISTRY
        .register(Box::new(FAILS_ON_UPDATES_SUBMISSION.clone()))
        .expect("fails_on_updates_submission can't be registered");

    REGISTRY
        .register(Box::new(CHAIN_EXECUTION_BLOCK_HEIGHT_ON_ETH.clone()))
        .expect("chain_execution_block_height_on_eth can't be registered");

    REGISTRY
        .register(Box::new(
            CHAIN_FINALIZED_EXECUTION_BLOCK_HEIGHT_ON_ETH.clone(),
        ))
        .expect("chain_finalized_execution_block_height_on_eth can't be registered");

    REGISTRY
        .register(Box::new(CHAIN_EXECUTION_BLOCK_HEIGHT_ON_NEAR.clone()))
        .expect("chain_execution_block_height_on_near can't be registered");

    REGISTRY
        .register(Box::new(
            CHAIN_FINALIZED_EXECUTION_BLOCK_HEIGHT_ON_NEAR.clone(),
        ))
        .expect("chain_finalized_execution_block_height_on_near can't be registered");
}

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {:?}", e);
    };
    let mut res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        eprintln!("could not encode prometheus metrics: {:?}", e);
    };
    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };

    res.push_str(&res_custom);
    Ok(res)
}

pub fn run_prometheus_service(port: u16) {
    register_custom_metrics();

    let metrics_route = warp::path!("metrics").and_then(metrics_handler);

    let rt = Runtime::new().expect("Error on creating runtime for Prometheus service");
    let handle = rt.handle();

    println!("Started on port {}", port);
    handle.block_on(warp::serve(metrics_route).run(([0, 0, 0, 0], port)));
}

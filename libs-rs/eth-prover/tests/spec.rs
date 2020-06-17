mod utils;
use utils::{new_root, RuntimeStandalone, ExternalUser};

fn setup_factory() -> (RuntimeStandalone, ExternalUser) {
    let (mut r, near) = new_root("near".into());
    near.init_eth_client(&mut r, "eth-client".to_string(), true).unwrap();
    near.init_eth_prover(&mut r, "eth-prover".to_string(), "eth-client").unwrap();
    (r, near)
}

#[test]
fn block_hash_safe_from_eth_client() {
    let _ = setup_factory();
}
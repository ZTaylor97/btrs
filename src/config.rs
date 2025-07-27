use once_cell::sync::Lazy;
use rand::{Rng, distr::Alphanumeric};
use serde::Deserialize;
use std::sync::Arc;
use urlencoding::encode_binary;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub peer_id: String,
    pub max_peers: usize,
    pub port: usize,
}

static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    let cfg = Config::default();
    Arc::new(cfg)
});

impl Default for Config {
    fn default() -> Self {
        let prefix = b"-RS0001-";
        let mut peer_id_bytes = [0u8; 20];

        peer_id_bytes[..8].copy_from_slice(prefix);

        let rand_part: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        peer_id_bytes[8..].copy_from_slice(rand_part.as_bytes());

        let peer_id = encode_binary(&peer_id_bytes).into_owned();
        Self {
            peer_id,
            max_peers: 10,
            port: 6882,
        }
    }
}

// In your app or async code:
pub fn get_config() -> Arc<Config> {
    CONFIG.clone()
}

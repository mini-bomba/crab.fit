use std::{env, net::IpAddr};

use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicKeyExtractor {
    Peer(PeerIpKeyExtractor),
    Smart(SmartIpKeyExtractor),
}

impl DynamicKeyExtractor {
    pub fn from_env() -> Self {
        if env::var("BEHIND_PROXY").is_ok() || env::var("LISTEN_ADDR").is_ok_and(|a| a.starts_with("unix:")) {
            Self::Smart(SmartIpKeyExtractor)
        } else {
            Self::Peer(PeerIpKeyExtractor)
        }
    }
}

impl KeyExtractor for DynamicKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, tower_governor::GovernorError> {
        match self {
            Self::Peer(inner) => inner.extract(req),
            Self::Smart(inner) => inner.extract(req),
        }
    }
}

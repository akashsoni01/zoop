//! Build-time secrets from `secrets.toml` (gitignored).

include!(concat!(env!("OUT_DIR"), "/secrets_config.rs"));

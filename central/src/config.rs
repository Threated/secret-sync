use std::{net::SocketAddr, convert::Infallible};

use beam_lib::{AppId, reqwest::Url};
use clap::Parser;
use shared::{SecretResult, OIDCConfig};

use crate::keycloak::{KeyCloakConfig, self};

/// Central secret sync
#[derive(Debug, Parser)]
pub struct Config {
    /// Address the server should bind to
    #[clap(env, long, default_value = "0.0.0.0:8080")]
    pub bind_addr: SocketAddr,

    /// Url of the local beam proxy which is required to have sockets enabled
    #[clap(env, long, default_value = "http://beam-proxy:8081")]
    pub beam_url: Url,

    /// Beam api key
    #[clap(env, long)]
    pub beam_secret: String,

    /// The app id of this application
    #[clap(long, env, value_parser=|id: &str| Ok::<_, Infallible>(AppId::new_unchecked(id)))]
    pub beam_id: AppId,
}

#[derive(Parser, Clone, Debug)]
pub struct GitlabConfig {
    pub gitlab_url: String,
    pub private_token: String,
}

#[derive(Clone, Debug)]
pub enum OIDCProvider {
    Keycloak(KeyCloakConfig)
}

impl OIDCProvider {
    pub fn try_init() -> Option<Self> {
        KeyCloakConfig::try_parse().map_err(|e| println!("{e}")).ok().map(Self::Keycloak)
    }

    pub async fn create_client(&self, name: &str, oidc_client_config: OIDCConfig) -> Result<SecretResult, String> {
        match self {
            OIDCProvider::Keycloak(conf) => keycloak::create_client(name, oidc_client_config, conf).await,
        }.map_err(|e| {
            println!("Failed to create client: {e}");
            "Error creating OIDC client".into()
        })
    }

    pub async fn validate_client(&self, name: &str, secret: &str, oidc_client_config: &OIDCConfig) -> Result<bool, String> {
        match self {
            OIDCProvider::Keycloak(conf) => {
                keycloak::validate_client(name, oidc_client_config, secret, conf)
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to validate client {name}: {e}");
                        "Failed to validate client. See upstrean logs.".into()
                    })
            },
        }
    }
}

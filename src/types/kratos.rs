use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub traits: Traits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Traits {
    pub email: String,
    pub providers: Vec<Provider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub provider_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityOidc {
    pub credentials: Credentials,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub oidc: Oidc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oidc {
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<ConfigProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigProvider {
    pub subject: String,
    pub provider: String,
    pub initial_access_token: String,
}

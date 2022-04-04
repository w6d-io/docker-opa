use serde::{Deserialize, Serialize};
use crate::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opadata {
    pub id: String,
    pub traits: Traits,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Traits {
    pub email: String,
    pub name: Name,
    pub roles: Roles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Roles {
    pub organizations: Vec<Value>,
    #[serde(rename = "private_projects")]
    pub private_projects: Vec<PrivateProject>,
    pub scopes: Vec<Value>,
    #[serde(rename = "affiliate_projects")]
    pub affiliate_projects: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateProject {
    pub key: i64,
    pub value: String,
}
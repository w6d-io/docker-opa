use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateProject {
    pub key: i64,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Name {
    UserName{ first: String, last: String},
    OrganizationName(String),
}

impl Default for Name {
    fn default() -> Self { Name::UserName{ first: String::default(), last: String::default()}}
}


#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Roles {
    pub organizations: Vec<Value>,
    #[serde(rename = "private_projects")]
    pub private_projects: Vec<PrivateProject>,
    pub scopes: Vec<Value>,
    #[serde(rename = "affiliate_projects")]
    pub affiliate_projects: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Traits {
    pub email: Option<String>,
    pub name: Name,
    pub roles: Option<Roles>,
    pub projects: Option<Vec<i32>> 
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opadata {
    pub id: String,
    pub traits: Traits,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

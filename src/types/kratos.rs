use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateProject {
    pub key: i64,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Name {
    UserName{ first: String, last: String},
    OrganizationName(String),
}

impl Default for Name {
    fn default() -> Self { Name::UserName{ first: String::default(), last: String::default()}}
}


#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roles {
    pub organizations: Vec<Value>,
    pub private_projects: Vec<PrivateProject>,
    pub scopes: Vec<Value>,
    pub affiliate_projects: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Traits {
    pub email: Option<String>,
    pub name: Name,
    pub roles: Option<Roles>,
    pub projects: Option<Vec<i32>> 
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetadataAdmin{
    pub roles: Option<Roles>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OpaData {
    pub id: String,
    pub traits: Traits,
    pub created_at: String,
    pub updated_at: String,
    pub metadata_admin: MetadataAdmin,
}

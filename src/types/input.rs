use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub eval: String,
    pub kratos: String,
    pub resource: i64,
    pub method: String,
    pub uri: String,
}

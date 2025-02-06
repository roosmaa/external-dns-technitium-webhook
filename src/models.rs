use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filters {
    pub filters: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Endpoint {
    #[serde(rename = "dnsName")]
    pub dns_name: String,
    pub targets: Vec<String>,
    #[serde(rename = "recordType")]
    pub record_type: String,
    #[serde(rename = "recordTTL")]
    pub record_ttl: Option<u32>,
    #[serde(rename = "setIdentifier")]
    pub set_identifier: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    #[serde(rename = "providerSpecific")]
    pub provider_specific: Option<Vec<ProviderSpecificProperty>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderSpecificProperty {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Endpoints {
    pub endpoints: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Changes {
    pub create: Option<Vec<Endpoint>>,
    #[serde(rename = "updateOld")]
    pub update_old: Option<Vec<Endpoint>>,
    #[serde(rename = "updateNew")]
    pub update_new: Option<Vec<Endpoint>>,
    pub delete: Option<Vec<Endpoint>>,
}

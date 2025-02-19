use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

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
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "recordTTL")]
    pub record_ttl: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "setIdentifier")]
    pub set_identifier: String,
    #[serde(skip_serializing_if = "is_default")]
    pub labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "providerSpecific")]
    pub provider_specific: Vec<ProviderSpecificProperty>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_endpoint_serialization() {
        let endpoint = Endpoint {
            dns_name: "example.com".to_string(),
            targets: vec!["1.2.3.4".to_string()],
            record_type: "A".to_string(),
            record_ttl: Some(300),
            ..Default::default()
        };
        let serialized = serde_json::to_value(&endpoint).unwrap();
        let expected = json!({
            "dnsName": "example.com",
            "targets": ["1.2.3.4"],
            "recordType": "A",
            "recordTTL": 300,
        });
        assert_eq!(serialized, expected);
    }
}

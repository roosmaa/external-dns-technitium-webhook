use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filters {
    pub filters: Vec<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Default)]
pub struct Endpoint {
    #[serde(rename = "dnsName")]
    pub dns_name: String,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(rename = "recordType")]
    pub record_type: String,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(rename = "recordTTL")]
    pub record_ttl: Option<u32>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    #[serde(rename = "setIdentifier")]
    pub set_identifier: String,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    #[serde(rename = "providerSpecific")]
    pub provider_specific: Vec<ProviderSpecificProperty>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct ProviderSpecificProperty {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Changes {
    #[serde(alias = "Create")]
    pub create: Option<Vec<Endpoint>>,
    #[serde(rename = "updateOld", alias = "UpdateOld")]
    pub update_old: Option<Vec<Endpoint>>,
    #[serde(rename = "updateNew", alias = "UpdateNew")]
    pub update_new: Option<Vec<Endpoint>>,
    #[serde(alias = "Delete")]
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

    #[test]
    fn test_endpoint_deserialization() {
        let json = json!({
            "dnsName": "example.com",
            "targets": ["1.2.3.4"],
            "recordType": "A",
            "recordTTL": 300,
        });
        let endpoint: Endpoint = serde_json::from_value(json).unwrap();
        assert_eq!(endpoint.dns_name, "example.com");
        assert_eq!(endpoint.targets, vec!["1.2.3.4".to_string()]);
        assert_eq!(endpoint.record_type, "A");
        assert_eq!(endpoint.record_ttl, Some(300));
    }

    #[test]
    fn test_changes_deserialization() {
        let json = json!({
            "Create": [{
                "dnsName": "example.com",
                "targets": ["1.2.3.4"],
                "recordType": "A",
                "recordTTL": 300,
            }],
            "UpdateOld": null,
            "UpdateNew": null,
            "Delete": null,
        });
        let changes: Changes = serde_json::from_value(json).unwrap();
        let create = changes.create.unwrap_or_default();
        assert_eq!(create.len(), 1);
        let endpoint = &create[0];
        assert_eq!(endpoint.dns_name, "example.com");
        assert_eq!(endpoint.targets, vec!["1.2.3.4".to_string()]);
        assert_eq!(endpoint.record_type, "A");
        assert_eq!(endpoint.record_ttl, Some(300));
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Default)]
pub struct LoginPayload {
    /// The username for the user account. The built-in administrator username on the DNS server is `admin`.
    #[serde(rename = "user")]
    pub username: String,
    /// The password for the user account. The default password for `admin` user is `admin`.
    #[serde(rename = "pass")]
    pub password: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct LoginResponse {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub username: String,
    pub token: String,
}

#[derive(Debug, Serialize, Default)]
pub struct CreateZonePayload {
    pub zone: String,
    #[serde(rename = "type")]
    pub zone_type: ZoneType,
    pub protocol: Option<Protocol>,
    pub forwarder: Option<String>,
    #[serde(rename = "dnssecValidation")]
    pub dnssec_validation: Option<bool>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct CreateZoneResponse {
    pub domain: String,
}

#[derive(Debug, Serialize, Default)]
pub struct ListZonesPayload {
    pub zone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pageNumber")]
    pub page_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "zonesPerPage")]
    pub zones_per_page: Option<u32>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ListZonesResponse {
    #[serde(rename = "pageNumber")]
    pub page_number: u32,
    #[serde(rename = "totalPages")]
    pub total_pages: u32,
    #[serde(rename = "totalZones")]
    pub total_zones: u32,
    pub zones: Vec<ZoneInfo>,
}

#[derive(Debug, Serialize, Default)]
pub struct AddRecordPayload {
    #[serde(rename = "domain")]
    pub domain: String,
    #[serde(flatten)]
    pub data: AddRecordPayloadRecordData,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "zone")]
    pub zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ttl")]
    pub ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "overwrite")]
    pub overwrite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "comments")]
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "expiryTtl")]
    pub expiry_ttl: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum AddRecordPayloadRecordData {
    #[serde(rename = "A")]
    A(RecordAData),
    #[serde(rename = "AAAA")]
    AAAA(RecordAAAAData),
    #[serde(rename = "CNAME")]
    CNAME(RecordCNAMEData),
    #[serde(rename = "TXT")]
    TXT(RecordTXTData),
    #[serde(untagged)]
    Other {
        #[serde(rename = "type")]
        record_type: String,
        #[serde(rename = "rdata")]
        data: serde_json::Value,
    },
}

impl Default for AddRecordPayloadRecordData {
    fn default() -> Self {
        return Self::Other {
            record_type: "".to_string(),
            data: serde_json::Value::String("".to_string()),
        };
    }
}

impl From<RecordAData> for AddRecordPayloadRecordData {
    fn from(value: RecordAData) -> Self {
        AddRecordPayloadRecordData::A(value)
    }
}

impl From<RecordAAAAData> for AddRecordPayloadRecordData {
    fn from(value: RecordAAAAData) -> Self {
        AddRecordPayloadRecordData::AAAA(value)
    }
}

impl From<RecordCNAMEData> for AddRecordPayloadRecordData {
    fn from(value: RecordCNAMEData) -> Self {
        AddRecordPayloadRecordData::CNAME(value)
    }
}

impl From<RecordTXTData> for AddRecordPayloadRecordData {
    fn from(value: RecordTXTData) -> Self {
        AddRecordPayloadRecordData::TXT(value)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct AddRecordResponse {
    pub zone: ZoneInfo,
    #[serde(rename = "addedRecord")]
    pub added_record: RecordInfo,
}

#[derive(Debug, Serialize, Default)]
pub struct GetRecordsPayload {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "listZone")]
    pub list_zone: Option<bool>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct GetRecordsResponse {
    pub zone: ZoneInfo,
    pub records: Vec<RecordInfo>,
}

#[derive(Debug, Serialize, Default)]
pub struct DeleteRecordPayload {
    #[serde(rename = "domain")]
    pub domain: String,
    #[serde(flatten)]
    pub data: DeleteRecordPayloadRecordData,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "zone")]
    pub zone: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum DeleteRecordPayloadRecordData {
    #[serde(rename = "A")]
    A(RecordAData),
    #[serde(rename = "AAAA")]
    AAAA(RecordAAAAData),
    #[serde(rename = "CNAME")]
    CNAME(RecordCNAMEData),
    #[serde(rename = "TXT")]
    TXT(RecordTXTData),
    #[serde(untagged)]
    Other {
        #[serde(rename = "type")]
        record_type: String,
        #[serde(rename = "rdata")]
        data: serde_json::Value,
    },
}

impl Default for DeleteRecordPayloadRecordData {
    fn default() -> Self {
        Self::Other {
            record_type: "".to_string(),
            data: serde_json::Value::String("".to_string()),
        }
    }
}

impl From<RecordAData> for DeleteRecordPayloadRecordData {
    fn from(data: RecordAData) -> Self {
        Self::A(data)
    }
}

impl From<RecordAAAAData> for DeleteRecordPayloadRecordData {
    fn from(data: RecordAAAAData) -> Self {
        Self::AAAA(data)
    }
}

impl From<RecordCNAMEData> for DeleteRecordPayloadRecordData {
    fn from(data: RecordCNAMEData) -> Self {
        Self::CNAME(data)
    }
}

impl From<RecordTXTData> for DeleteRecordPayloadRecordData {
    fn from(data: RecordTXTData) -> Self {
        Self::TXT(data)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct DeleteRecordResponse {}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ZoneInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub zone_type: ZoneType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    pub disabled: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct RecordInfo {
    pub disabled: bool,
    pub name: String,
    pub ttl: u32,
    #[serde(flatten)]
    pub data: RecordData,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "type", content = "rData")]
pub enum RecordData {
    #[serde(rename = "A")]
    A(RecordAData),
    #[serde(rename = "AAAA")]
    AAAA(RecordAAAAData),
    #[serde(rename = "CNAME")]
    CNAME(RecordCNAMEData),
    #[serde(rename = "TXT")]
    TXT(RecordTXTData),
    #[serde(untagged)]
    Other {
        #[serde(rename = "type")]
        record_type: String,
        #[serde(rename = "rData")]
        data: serde_json::Value,
    },
}

impl Default for RecordData {
    fn default() -> Self {
        RecordData::Other {
            record_type: "".to_string(),
            data: serde_json::Value::String("".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordAData {
    #[serde(rename = "ipAddress")]
    pub ip_address: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordAAAAData {
    #[serde(rename = "ipAddress")]
    pub ip_address: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordCNAMEData {
    #[serde(rename = "cname")]
    pub cname: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Default)]
pub struct RecordTXTData {
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordAUpdate {
    #[serde(flatten)]
    pub current: RecordAData,
    #[serde(rename = "newIpAddress")]
    pub ip_address: String,
}

impl From<RecordAData> for RecordAUpdate {
    fn from(data: RecordAData) -> Self {
        Self {
            current: data.clone(),
            ip_address: data.ip_address,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordAAAAUpdate {
    #[serde(flatten)]
    pub current: RecordAAAAData,
    #[serde(rename = "newIpAddress")]
    pub ip_address: String,
}

impl From<RecordAAAAData> for RecordAAAAUpdate {
    fn from(data: RecordAAAAData) -> Self {
        Self {
            current: data.clone(),
            ip_address: data.ip_address,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordCNAMEUpdate {
    #[serde(flatten)]
    pub current: RecordCNAMEData,
    #[serde(rename = "newCname")]
    pub cname: String,
}

impl From<RecordCNAMEData> for RecordCNAMEUpdate {
    fn from(data: RecordCNAMEData) -> Self {
        Self {
            current: data.clone(),
            cname: data.cname,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct RecordTXTUpdate {
    #[serde(flatten)]
    pub current: RecordTXTData,
    #[serde(rename = "newText")]
    pub text: String,
}

impl From<RecordTXTData> for RecordTXTUpdate {
    fn from(data: RecordTXTData) -> Self {
        Self {
            current: data.clone(),
            text: data.text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
pub enum ZoneType {
    #[default]
    #[serde(rename = "Primary")]
    Primary,
    #[serde(rename = "Secondary")]
    Secondary,
    #[serde(rename = "Stub")]
    Stub,
    #[serde(rename = "Forwarder")]
    Forwarder,
    #[serde(rename = "SecondaryForwarder")]
    SecondaryForwarder,
    #[serde(rename = "Catalog")]
    Catalog,
    #[serde(rename = "SecondaryCatalog")]
    SecondaryCatalog,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum Protocol {
    #[default]
    #[serde(rename = "Udp")]
    Udp,
    #[serde(rename = "Tcp")]
    Tcp,
    #[serde(rename = "Tls")]
    Tls,
    #[serde(rename = "Https")]
    Https,
    #[serde(rename = "Quic")]
    Quic,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_record_info_a_deserialization() {
        let data = json!({
            "disabled": false,
            "name": "example.com",
            "type": "A",
            "ttl": 3600,
            "rData": {
                "ipAddress": "1.1.1.1"
            },
        });

        let record: RecordInfo = serde_json::from_value(data).unwrap();
        assert!(!record.disabled);
        assert_eq!(record.name, "example.com");
        assert_eq!(record.ttl, 3600);

        if let RecordData::A(a_data) = record.data {
            assert_eq!(a_data.ip_address, "1.1.1.1");
        } else {
            panic!("Expected RecordData::A");
        }
    }

    #[test]
    fn test_record_info_cname_deserialization() {
        let data = json!({
            "disabled": false,
            "name": "www.example.com",
            "type": "CNAME",
            "ttl": 3600,
            "rData": {
                "cname": "example.com"
            },
        });

        let record: RecordInfo = serde_json::from_value(data).unwrap();
        assert!(!record.disabled);
        assert_eq!(record.name, "www.example.com");
        assert_eq!(record.ttl, 3600);

        if let RecordData::CNAME(cname_data) = record.data {
            assert_eq!(cname_data.cname, "example.com");
        } else {
            panic!("Expected RecordData::CNAME");
        }
    }

    #[test]
    fn test_record_info_txt_deserialization() {
        let data = json!({
            "disabled": false,
            "name": "example.com",
            "type": "TXT",
            "ttl": 3600,
            "rData": {
                "text": "v=spf1 include:example.com -all"
            },
        });

        let record: RecordInfo = serde_json::from_value(data).unwrap();
        assert!(!record.disabled);
        assert_eq!(record.name, "example.com");
        assert_eq!(record.ttl, 3600);

        if let RecordData::TXT(txt_data) = record.data {
            assert_eq!(txt_data.text, "v=spf1 include:example.com -all");
        } else {
            panic!("Expected RecordData::TXT");
        }
    }

    #[test]
    fn test_record_info_xxx_deserialization() {
        let data = json!({
            "disabled": false,
            "name": "sub.example.com",
            "type": "XXX",
            "ttl": 3600,
            "rData": "AA00",
        });

        let record: RecordInfo = serde_json::from_value(data).unwrap();
        assert!(!record.disabled);
        assert_eq!(record.name, "sub.example.com");
        assert_eq!(record.ttl, 3600);

        if let RecordData::Other { record_type, data } = record.data {
            assert_eq!(record_type, "XXX");
            assert_eq!(data.as_str(), Some("AA00"));
        } else {
            panic!("Expected RecordData::Unknown");
        }
    }

    #[test]
    fn test_add_record_payload_serialization_for_a_record() {
        let serialized = serde_urlencoded::to_string(&AddRecordPayload {
            domain: "example.com".to_string(),
            ttl: Some(3600),
            data: AddRecordPayloadRecordData::A(RecordAData {
                ip_address: "1.1.1.1".to_string(),
            }),
            ..Default::default()
        })
        .unwrap();

        let expected = "domain=example.com&type=A&ipAddress=1.1.1.1&ttl=3600";

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_add_record_payload_serialization_for_txt_record() {
        let serialized = serde_urlencoded::to_string(&AddRecordPayload {
            domain: "example.com".to_string(),
            ttl: Some(3600),
            data: AddRecordPayloadRecordData::TXT(RecordTXTData {
                text: "v=spf1 include:example.com -all".to_string(),
            }),
            ..Default::default()
        })
        .unwrap();

        let expected =
            "domain=example.com&type=TXT&text=v%3Dspf1+include%3Aexample.com+-all&ttl=3600";

        assert_eq!(serialized, expected);
    }
}

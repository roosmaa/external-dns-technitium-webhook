use super::models::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Debug, Error)]
pub enum TechnitiumError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
}

#[derive(Debug)]
pub struct TechnitiumClient {
    client: Client,
    base_url: String,
    token: String,
}

impl TechnitiumClient {
    const ENDPOINT_LOGIN: &'static str = "/api/user/login";
    const ENDPOINT_CREATE_ZONE: &'static str = "/api/zones/create";
    const ENDPOINT_LIST_ZONES: &'static str = "/api/zones/list";
    const ENDPOINT_ADD_RECORD: &'static str = "/api/zones/records/add";
    const ENDPOINT_GET_RECORDS: &'static str = "/api/zones/records/get";
    const ENDPOINT_DELETE_RECORD: &'static str = "/api/zones/records/delete";

    pub fn new(base_url: String, token: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            base_url,
            token,
        }
    }

    #[inline]
    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        payload: T,
    ) -> Result<R, TechnitiumError> {
        let req = AuthenticatedRequest {
            token: &self.token,
            payload,
        };
        let resp: NestedResponse<R> = self.post_raw(endpoint, req).await?;
        Ok(resp.response)
    }

    async fn post_raw<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: T,
    ) -> Result<R, TechnitiumError> {
        let url = format!("{}{}", self.base_url, endpoint);
        debug!("Sending POST request to {}", url);

        let response = self.client.post(&url).form(&params).send().await?;

        if !response.status().is_success() {
            return Err(TechnitiumError::ApiError(format!(
                "Server responded with non-successful status code {}",
                response.status()
            )));
        }

        let response: Response<R> = response.json().await?;
        match response.status {
            ResponseStatus::Ok => response
                .data
                .ok_or_else(|| TechnitiumError::ApiError("Missing response data".to_string())),
            ResponseStatus::Error => Err(TechnitiumError::ApiError(
                response
                    .error_message
                    .unwrap_or("Unknown server error".to_string()),
            )),
            ResponseStatus::InvalidToken => {
                Err(TechnitiumError::ApiError("Invalid token".to_string()))
            }
            ResponseStatus::Unknown(status) => Err(TechnitiumError::ApiError(format!(
                "Unexpected response status ({})",
                status
            ))),
        }
    }

    #[inline]
    pub async fn login(&self, payload: LoginPayload) -> Result<LoginResponse, TechnitiumError> {
        self.post_raw(Self::ENDPOINT_LOGIN, payload).await
    }

    #[inline]
    pub async fn create_zone(
        &self,
        payload: CreateZonePayload,
    ) -> Result<CreateZoneResponse, TechnitiumError> {
        self.post(Self::ENDPOINT_CREATE_ZONE, payload).await
    }

    #[inline]
    pub async fn list_zones(
        &self,
        payload: ListZonesPayload,
    ) -> Result<ListZonesResponse, TechnitiumError> {
        self.post(Self::ENDPOINT_LIST_ZONES, payload).await
    }

    #[inline]
    pub async fn add_record(
        &self,
        payload: AddRecordPayload,
    ) -> Result<AddRecordResponse, TechnitiumError> {
        self.post(Self::ENDPOINT_ADD_RECORD, payload).await
    }

    #[inline]
    pub async fn get_records(
        &self,
        payload: GetRecordsPayload,
    ) -> Result<GetRecordsResponse, TechnitiumError> {
        self.post(Self::ENDPOINT_GET_RECORDS, payload).await
    }

    #[inline]
    pub async fn delete_record(
        &self,
        payload: DeleteRecordPayload,
    ) -> Result<DeleteRecordResponse, TechnitiumError> {
        self.post(Self::ENDPOINT_DELETE_RECORD, payload).await
    }
}

#[derive(Serialize)]
struct AuthenticatedRequest<'a, T: Serialize> {
    token: &'a str,
    #[serde(flatten)]
    payload: T,
}

#[derive(Debug, Deserialize)]
struct Response<T> {
    status: ResponseStatus,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
    #[serde(flatten)]
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct NestedResponse<T> {
    response: T,
}

#[derive(Debug, Deserialize)]
enum ResponseStatus {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "invalid-token")]
    InvalidToken,
    #[serde(untagged)]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_client_login() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "status": "ok",
            "displayName": "Administrator",
            "username": "admin",
            "token": "sample_token",
        });

        let mock = server
            .mock("POST", "/api/user/login")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("user".into(), "admin".into()),
                mockito::Matcher::UrlEncoded("pass".into(), "password123".into()),
            ]))
            .with_status(200)
            .with_body(response_data.to_string())
            .create();

        let client = TechnitiumClient::new(
            server.url(),
            "dummy_token".to_string(),
            Duration::from_secs(30),
        );
        let res = client
            .login(LoginPayload {
                username: "admin".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        mock.assert();
        assert_eq!(
            res,
            LoginResponse {
                display_name: "Administrator".to_string(),
                username: "admin".to_string(),
                token: "sample_token".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_client_add_zone() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "status": "ok",
            "response": {
                "domain": "example.com",
            },
        });

        let mock = server
            .mock("POST", "/api/zones/create")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("zone".into(), "example.com".into()),
                mockito::Matcher::UrlEncoded("type".into(), "Primary".into()),
            ]))
            .with_status(200)
            .with_body(response_data.to_string())
            .create();

        let client =
            TechnitiumClient::new(server.url(), "token".to_string(), Duration::from_secs(30));
        let res = client
            .create_zone(CreateZonePayload {
                zone: "example.com".to_string(),
                zone_type: ZoneType::Primary,
                ..Default::default()
            })
            .await
            .unwrap();

        mock.assert();
        assert_eq!(
            res,
            CreateZoneResponse {
                domain: "example.com".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_client_add_record() {
        let mut server = mockito::Server::new_async().await;

        let response_data = json!({
            "response": {
                "zone": {
                    "name": "example.com",
                    "type": "Primary",
                    "internal": false,
                    "dnssecStatus": "SignedWithNSEC",
                    "disabled": false
                },
                "addedRecord": {
                    "disabled": false,
                    "name": "example.com",
                    "type": "A",
                    "ttl": 3600,
                    "rData": {
                        "ipAddress": "3.3.3.3"
                    },
                }
            },
            "status": "ok"
        });

        let mock = server
            .mock("POST", "/api/zones/records/add")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("domain".into(), "example.com".into()),
                mockito::Matcher::UrlEncoded("ttl".into(), "3600".into()),
                mockito::Matcher::UrlEncoded("type".into(), "A".into()),
                mockito::Matcher::UrlEncoded("ipAddress".into(), "3.3.3.3".into()),
            ]))
            .with_status(200)
            .with_body(response_data.to_string())
            .create();

        let client =
            TechnitiumClient::new(server.url(), "token".to_string(), Duration::from_secs(30));
        let res = client
            .add_record(AddRecordPayload {
                domain: "example.com".to_string(),
                ttl: Some(3600),
                data: AddRecordPayloadRecordData::A(RecordAData {
                    ip_address: "3.3.3.3".to_string(),
                }),
                ..Default::default()
            })
            .await
            .unwrap();

        mock.assert();
        assert_eq!(
            res,
            AddRecordResponse {
                zone: ZoneInfo {
                    name: "example.com".to_string(),
                    zone_type: ZoneType::Primary,
                    internal: Some(false),
                    disabled: false,
                },
                added_record: RecordInfo {
                    disabled: false,
                    name: "example.com".to_string(),
                    ttl: 3600,
                    data: RecordData::A(RecordAData {
                        ip_address: "3.3.3.3".to_string(),
                    }),
                },
            }
        );
    }
}

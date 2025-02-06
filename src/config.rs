use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_address: String,
    pub listen_port: String,
    pub technitium_url: String,
    pub technitium_username: String,
    pub technitium_password: String,
    pub zone: String,
    pub domain_filters: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".to_string(),
            listen_port: "3000".to_string(),
            technitium_url: String::new(),
            technitium_username: String::new(),
            technitium_password: String::new(),
            zone: String::new(),
            domain_filters: None,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen_address: env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            listen_port: env::var("LISTEN_PORT").unwrap_or_else(|_| "3000".to_string()),
            technitium_url: env::var("TECHNITIUM_URL").expect("Missing TECHNITIUM_URL"),
            technitium_username: env::var("TECHNITIUM_USERNAME")
                .expect("Missing TECHNITIUM_USERNAME"),
            technitium_password: env::var("TECHNITIUM_PASSWORD")
                .expect("Missing TECHNITIUM_PASSWORD"),
            zone: env::var("ZONE").expect("Missing ZONE"),
            domain_filters: env::var("DOMAIN_FILTERS")
                .ok()
                .map(|v| v.split(';').map(String::from).collect()),
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.listen_address, self.listen_port)
    }
}

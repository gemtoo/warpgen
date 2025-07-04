use reqwest::header;
use serde_json::json;
use tracing::{debug, error, info, instrument};

use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    result: ResultData,
}

#[derive(Deserialize)]
struct ResultData {
    config: Config,
}

#[derive(Deserialize)]
struct Config {
    interface: Interface,
    peers: Vec<Peer>,
}

#[derive(Deserialize)]
struct Interface {
    addresses: Addresses,
}

#[derive(Deserialize)]
struct Addresses {
    v4: String,
    v6: String,
}

#[derive(Deserialize)]
struct Peer {
    public_key: String,
    endpoint: Endpoint,
}

#[derive(Deserialize)]
struct Endpoint {
    host: String,
}

#[derive(Debug, Default)]
pub struct WarpConfig {
    private_key: String,
    public_key: String,
    address_v4: String,
    address_v6: String,
    peer_public_key: String,
    endpoint: String,
}

impl WarpConfig {
    #[instrument(level = "info", skip_all)]
    pub async fn generate() -> Result<String, Box<dyn std::error::Error>> {
        let mut config = WarpConfig::default();
        config.generate_keys().await?;
        config.fetch_configuration().await?;
        config.format_config()
    }

    #[instrument(level = "info", skip(self))]
    async fn generate_keys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Generating new WireGuard keys");
        self.private_key = wireguard_keys::Privkey::generate().to_base64();
        debug!("Generated private key: {}", self.private_key);

        let privkey_obj: wireguard_keys::Privkey = self.private_key.parse().map_err(|e| {
            error!("Failed to parse private key: {}", e);
            e
        })?;
        self.public_key = privkey_obj.pubkey().to_base64();
        info!("Derived public key from private key");
        debug!("Derived public key: {}", self.public_key);
        
        Ok(())
    }

    #[instrument(level = "info", skip(self))]
    async fn fetch_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let api_url = "https://api.cloudflareclient.com/v0i1909051800/reg";
        info!(url = api_url, "Preparing API request");

        let tos_time = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();
        debug!("Generated TOS timestamp: {}", tos_time);

        let payload = json!({
            "install_id": "",
            "tos": tos_time,
            "key": &self.public_key,
            "fcm_token": "",
            "type": "ios",
            "locale": "en_US"
        });
        debug!(payload = %payload, "Request payload");

        let mut headers = header::HeaderMap::new();
        headers.insert("user-agent", header::HeaderValue::from_static(""));
        headers.insert(
            "content-type",
            header::HeaderValue::from_static("application/json"),
        );

        let response = send_request(&client, api_url, headers, payload).await?;
        let status = response.status();
        let json_body = response.text().await?;
        info!(%status, "Received API response");
        debug!("Response body: {}", json_body);

        let response: Response = serde_json::from_str(&json_body)?;
        self.address_v4 = response.result.config.interface.addresses.v4;
        self.address_v6 = response.result.config.interface.addresses.v6;

        let peer = response
            .result
            .config
            .peers
            .first()
            .ok_or("No peers found in configuration")?;
        self.peer_public_key = peer.public_key.clone();
        self.endpoint = peer.endpoint.host.clone();

        Ok(())
    }

    #[instrument(level = "info", skip(self))]
    fn format_config(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();

        // Interface section
        output.push_str("[Interface]\n");
        output.push_str(&format!("PrivateKey = {}\n", self.private_key));
        output.push_str(&format!("Address = {}, {}\n", self.address_v4, self.address_v6));
        output.push_str("DNS = 1.1.1.1, 2606:4700:4700::1111, 1.0.0.1, 2606:4700:4700::1001\n");

        // Peer section
        output.push_str("\n[Peer]\n");
        output.push_str(&format!("PublicKey = {}\n", self.peer_public_key));
        output.push_str("AllowedIPs = 0.0.0.0/0, ::/0\n");
        output.push_str(&format!("Endpoint = {}\n", self.endpoint));

        Ok(output)
    }
}

#[instrument(
    name = "api_request",
    level = "info",
    skip(client, headers, payload),
    fields(
        url,
        method = "POST",
        has_headers = !headers.is_empty()
    )
)]
async fn send_request(
    client: &reqwest::Client,
    url: &str,
    headers: header::HeaderMap,
    payload: serde_json::Value,
) -> Result<reqwest::Response, reqwest::Error> {
    tracing::Span::current().record("url", url);

    client
        .post(url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map(|res| {
            debug!("HTTP request dispatched");
            res
        })
        .map_err(|err| {
            error!("HTTP request failed: {}", err);
            err
        })
}

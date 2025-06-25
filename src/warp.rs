use reqwest::header;
use serde_json::json;
use tracing::{debug, error, info, instrument};

use serde::Deserialize;
use std::error::Error;
use std::fmt::Write;

#[derive(Deserialize)]
struct Response {
    result: ResultData,
}

#[derive(Deserialize)]
struct ResultData {
    key: String,
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

pub async fn generate() -> Result<String, Box<dyn std::error::Error>> {
    info!("Generating new WireGuard keys");
    let privkey = wireguard_keys::Privkey::generate().to_base64();
    debug!("Generated private key: {}", privkey);

    let privkey_obj: wireguard_keys::Privkey = privkey.parse().map_err(|e| {
        error!("Failed to parse private key: {}", e);
        e
    })?;
    let pubkey = privkey_obj.pubkey().to_base64();
    info!("Derived public key from private key");
    debug!("Derived public key: {}", pubkey);

    // Prepare API request
    let client = reqwest::Client::new();
    let api_url = "https://api.cloudflareclient.com/v0i1909051800/reg";
    info!(url = api_url, "Preparing API request");

    // Create ISO 8601 timestamp (UTC)
    let tos_time = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S.000Z")
        .to_string();
    debug!("Generated TOS timestamp: {}", tos_time);

    // Build JSON payload
    let payload = json!({
        "install_id": "",
        "tos": tos_time,
        "key": pubkey,
        "fcm_token": "",
        "type": "ios",
        "locale": "en_US"
    });
    debug!(payload = %payload, "Request payload");

    // Create request headers
    let mut headers = header::HeaderMap::new();
    headers.insert("user-agent", header::HeaderValue::from_static(""));
    headers.insert(
        "content-type",
        header::HeaderValue::from_static("application/json"),
    );

    // Instrumented HTTP request
    let response = send_request(&client, api_url, headers, payload).await?;

    // Output response
    let status = response.status();
    let json_body = response.text().await?;
    info!(%status, "Received API response");
    debug!("Response body: {}", json_body);

    let response: Response = serde_json::from_str(&json_body)?;
    
    let private_key = privkey;
    let address_v4 = response.result.config.interface.addresses.v4;
    let address_v6 = response.result.config.interface.addresses.v6;
    
    let peer = response.result.config.peers.first()
        .ok_or("No peers found in configuration")?;
    let public_key = &peer.public_key;
    let endpoint = &peer.endpoint.host;

    let mut output = String::new();
    
    // Interface section
    writeln!(&mut output, "[Interface]")?;
    writeln!(&mut output, "PrivateKey = {}", private_key)?;
    writeln!(&mut output, "Address = {}, {}", address_v4, address_v6)?;
    writeln!(&mut output, "DNS = 1.1.1.1, 2606:4700:4700::1111, 1.0.0.1, 2606:4700:4700::1001")?;
    
    // Peer section
    writeln!(&mut output, "\n[Peer]")?;
    writeln!(&mut output, "PublicKey = {}", public_key)?;
    writeln!(&mut output, "AllowedIPs = 0.0.0.0/0, ::/0")?;
    writeln!(&mut output, "Endpoint = {}", endpoint)?;

    Ok(output)
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

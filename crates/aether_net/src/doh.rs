use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Deserialize, Debug)]
struct DohResponse {
    #[serde(rename = "Status")]
    _status: i32,
    #[serde(rename = "Answer")]
    answer: Option<Vec<DohAnswer>>,
}

#[derive(Deserialize, Debug)]
struct DohAnswer {
    data: String,
}

pub struct DohClient {
    client: Client,
    endpoint: String,
    // [Category C] DNS Rebinding: Alternates between real and target IPs
    rebinding_counter: AtomicUsize,
}

impl DohClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "https://cloudflare-dns.com/dns-query".to_string(),
            rebinding_counter: AtomicUsize::new(0),
        }
    }

    /// Resolves a domain using DoH with optional Rebinding logic.
    pub async fn resolve(&self, domain: &str, rebind_target: Option<String>) -> Result<String> {
        // [Category C] DNS Rebinding: If a target IP is provided, alternate results.
        if let Some(target) = rebind_target {
            let count = self.rebinding_counter.fetch_add(1, Ordering::Relaxed);
            if count % 2 == 1 {
                return Ok(target);
            }
        }

        let url = format!("{}?name={}&type=A", self.endpoint, domain);
        let resp = self.client.get(&url)
            .header("accept", "application/dns-json")
            .send()
            .await?
            .json::<DohResponse>()
            .await?;

        if let Some(answers) = resp.answer {
            if let Some(first) = answers.first() {
                return Ok(first.data.clone());
            }
        }
        
        Err(anyhow::anyhow!("No DNS record found"))
    }
}

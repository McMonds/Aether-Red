use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

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
}

impl DohClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "https://cloudflare-dns.com/dns-query".to_string(),
        }
    }

    pub async fn resolve(&self, domain: &str) -> Result<String> {
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

use reqwest::Url;
use crate::error::CaserError;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct CaserClient
{
    pub api_key: String,
    pub host: Url,
    pub reqwest_client: reqwest::Client,
}

impl CaserClient
{
    pub fn new(host: &str, api_key: String) -> Result<CaserClient, CaserError>
    {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(&api_key)?);

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        let host = Url::parse(&host)?;

        Ok(
            CaserClient {
                api_key,
                host,
                reqwest_client: client,
            }
        )
    }
}
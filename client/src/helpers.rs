use reqwest::Url;
use crate::error::CaserError;

pub struct UrlBuilder
{
    base: Url,
    parts: Vec<String>,
    query: Vec<(String, String)>,
}

impl UrlBuilder
{
    pub fn new(base: Url) -> Self
    {
        UrlBuilder {
            base,
            parts: vec![],
            query: vec![]
        }
    }

    pub fn add_part(&mut self, part: &str) -> &mut Self
    {
        self.parts.push(part.to_owned());
        self
    }

    pub fn add_query(&mut self, key: &str, value: &str) -> &mut Self
    {
        self.query.push((key.to_owned(), value.to_owned()));
        self
    }

    pub fn build(&mut self) -> Result<Url, CaserError>
    {
        let parts = self.parts.join("/");

        let mut result = parts;
        if self.query.len() > 0
        {
            let query: String = self.query
                .iter()
                .map(|(key, value)| key.to_owned() + "=" + value)
                .collect::<Vec<String>>()
                .join("&");

            result += &query;
        }

        Ok(
            self.base.join(&result)?
        )
    }
}
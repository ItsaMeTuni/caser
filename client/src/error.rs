use reqwest::Error;
use caser_common::event::FromPlainError;

#[derive(Error, Debug)]
pub enum CaserError
{
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    FromPlainError(#[from] FromPlainError),

    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}
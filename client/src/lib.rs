use std::future::Future;
use uuid::Uuid;
use caser_common::event::{Event, EventPlain};
use crate::error::CaserError;
use reqwest::header::{HeaderMap, HeaderValue};
use std::ops::Deref;
use caser_common::calendar::Calendar;
use reqwest::Url;
use std::string::ParseError;
use std::convert::TryInto;

#[macro_use] extern crate async_trait;
#[macro_use] extern crate thiserror;


pub mod error;
pub mod client;
pub mod calendar;
pub mod event;
mod helpers;
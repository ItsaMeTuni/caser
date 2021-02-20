use crate::client::CaserClient;
use caser_common::event::Event;
use std::ops::Deref;

pub struct CaserEvent<'client>
{
    pub client: &'client CaserClient,
    pub inner: Event,
}

impl<'client> Deref for CaserEvent<'client>
{
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
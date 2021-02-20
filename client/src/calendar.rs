use crate::client::CaserClient;
use caser_common::calendar::Calendar;
use uuid::Uuid;
use crate::error::CaserError;
use crate::event::CaserEvent;
use crate::helpers::UrlBuilder;
use caser_common::event::EventPlain;
use std::ops::Deref;
use std::convert::TryInto;

pub struct CaserCalendar<'client>
{
    client: &'client CaserClient,
    inner: Calendar,
}

impl<'client> CaserCalendar<'client>
{
    pub async fn get_event_by_id(&self, id: Uuid) -> Result<CaserEvent<'client>, CaserError>
    {
        let url = UrlBuilder::new(self.client.host.clone())
            .add_part("calendar")
            .add_part(&self.get_id().to_string())
            .add_part("events")
            .add_part(&id.to_string())
            .build()?;

        let req = self.client.reqwest_client.get(url).build()?;
        let response = self.client.reqwest_client.execute(req).await?;
        let event_plain: EventPlain = response.json().await?;

        Ok(
            CaserEvent {
                client: self.client,
                inner: event_plain.try_into()?
            }
        )
    }
}

impl<'client> Deref for CaserCalendar<'client>
{
    type Target = Calendar;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
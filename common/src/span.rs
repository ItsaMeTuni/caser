use chrono::{NaiveTime, NaiveDate, Duration, NaiveDateTime};

#[derive(Copy, Clone, Debug)]
pub struct EventDateSpan
{
    start: NaiveDate,
    end: NaiveDate,
}




#[derive(Copy, Clone, Debug)]
pub struct EventDateTimeSpan
{
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl EventDateTimeSpan
{
    pub fn as_date_span(&self) -> EventDateSpan
    {
        EventDateSpan {
            start: self.start.date(),
            end: self.end.date(),
        }
    }
}




#[derive(Copy, Clone, Debug)]
pub enum EventSpan
{
    Date(EventDateSpan),
    DateTime(EventDateTimeSpan),
}

impl EventSpan
{
    pub fn get_date_span(&self) -> EventDateSpan
    {
        match self
        {
            EventSpan::Date(date_span) => *date_span,
            EventSpan::DateTime(datetime_span) => datetime_span.as_date_span(),
        }
    }

    pub fn get_date_time_span(&self) -> Option<EventDateTimeSpan>
    {
        match self
        {
            EventSpan::Date(_) => None,
            EventSpan::DateTime(datetime_span) => Some(*datetime_span),
        }
    }

    pub fn get_start_date(&self) -> NaiveDate
    {
        self.get_date_span().start
    }

    pub fn get_end_date(&self) -> NaiveDate
    {
        self.get_date_span().end
    }

    pub fn get_start_time(&self) -> Option<NaiveTime>
    {
        self.get_date_time_span().map(|dt| dt.start.time())
    }

    pub fn get_end_time(&self) -> Option<NaiveTime>
    {
        self.get_date_time_span().map(|dt| dt.end.time())
    }

    pub fn get_duration(&self) -> Duration
    {
        match self
        {
            EventSpan::Date(date_span) => date_span.end - date_span.start,
            EventSpan::DateTime(datetime_span) => datetime_span.end - datetime_span.start,
        }
    }

    pub fn from_date_and_duration(start: NaiveDate, duration: Duration) -> EventSpan
    {
        EventSpan::Date(
            EventDateSpan {
                start,
                end: start + duration,
            }
        )
    }

    pub fn from_date_time_and_duration(start: NaiveDateTime, duration: Duration) -> EventSpan
    {
        EventSpan::DateTime(
            EventDateTimeSpan {
                start,
                end: start + duration,
            }
        )
    }
}
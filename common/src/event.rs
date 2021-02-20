//! A few notes:
//!
//! Everything is stored in UTC: `NaiveDate`s and `NaiveTime`s are all in UTC,
//! and the DATEs and TIMEs in the database are in UTC (and have no timezone).

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use crate::recurrence::RecurrenceRule;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::span::{EventSpan, EventDateTimeSpan, EventDateSpan};
use crate::recurrence::parser::RRuleParseError;
use std::convert::{TryFrom, TryInto};


#[derive(Clone, Debug)]
pub struct EventRecurrence
{
    rule: RecurrenceRule,
    exdates: Vec<NaiveDate>,
    rdates: Vec<NaiveDate>,
}

impl TryFrom<RecurrencePlain> for EventRecurrence
{
    type Error = FromPlainError;

    fn try_from(value: RecurrencePlain) -> Result<Self, Self::Error>
    {
        if value.rrule.is_none()
        {
            return Err(FromPlainError::MissingField)
        }

        Ok(
            EventRecurrence {
                rule: RecurrenceRule::new(&value.rrule.unwrap())
                    .map_err(|e| FromPlainError::RRuleParseError(e))?,
                rdates: value.rdates.unwrap_or(vec![]),
                exdates: value.exdates.unwrap_or(vec![]),
            }
        )
    }
}


#[derive(Debug)]
pub enum Event
{
    Recurring(EventRecurring),
    Single(EventSingle),
}

impl ToPlain<EventPlain> for Event
{
    fn into_plain(self) -> EventPlain
    {
        match self
        {
            Event::Recurring(e) => e.into_plain(),
            Event::Single(e) => e.into_plain(),
        }
    }
}

impl ToPlain<Vec<EventPlain>> for Vec<Event>
{
    fn into_plain(self) -> Vec<EventPlain>
    {
        self
            .into_iter()
            .map(|x| x.into_plain())
            .collect()
    }
}

impl TryFrom<EventPlain> for Event
{
    type Error = FromPlainError;

    fn try_from(value: EventPlain) -> Result<Self, Self::Error>
    {
        if value.start_date.is_none() || value.end_date.is_none()
        {
            return Err(FromPlainError::MissingField);
        }

        if value.start_time.is_some() != value.end_time.is_some()
        {
            return Err(FromPlainError::InvalidSpan);
        }

        if value.id.is_none()
        {
            return Err(FromPlainError::MissingField);
        }

        if value.last_modified.is_none()
        {
            return Err(FromPlainError::MissingField);
        }

        let span;
        if value.start_time.is_some()
        {
            span = EventSpan::DateTime(
                EventDateTimeSpan {
                    start: value.start_date.unwrap().and_time(value.start_time.unwrap()),
                    end: value.end_date.unwrap().and_time(value.end_time.unwrap())
                }
            );
        }
        else
        {
            span = EventSpan::Date(
                EventDateSpan {
                    start: value.start_date.unwrap(),
                    end: value.end_date.unwrap()
                }
            );
        }

        if value.recurrence.is_some()
        {
            Ok(
                Event::Recurring(
                    EventRecurring {
                        id: value.id.unwrap(),
                        span,
                        recurrence: value.recurrence.unwrap().try_into()?,
                        last_modified: value.last_modified.unwrap()
                    }
                )
            )
        }
        else
        {
            Ok(
                Event::Single(
                    EventSingle {
                        id: value.id.unwrap(),
                        parent_id: value.parent_id,
                        last_modified: value.last_modified.unwrap(),
                        span,
                    }
                )
            )
        }
    }
}




#[derive(Clone, Debug)]
pub struct EventRecurring
{
    /// Id of this event in the database.
    id: Uuid,
    span: EventSpan,
    recurrence: EventRecurrence,
    last_modified: NaiveDateTime,
}

/// If you want to get an event you have to get it from
/// its calendarold.
impl EventRecurring
{
    pub fn get_id(&self) -> Uuid { self.id }

    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_recurrence(&self) -> EventRecurrence { self.recurrence.clone() }

    pub fn get_last_modified(&self) -> NaiveDateTime { self.last_modified.clone() }
}

impl ToPlain<EventPlain> for EventRecurring
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: Some(self.id),
            parent_id: None,

            start_date: Some(self.span.get_start_date()),
            end_date: Some(self.span.get_end_date()),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: Some(
                RecurrencePlain {
                    rrule: Some(self.recurrence.rule.to_string()),
                    exdates: Some(self.recurrence.exdates),
                    rdates: Some(self.recurrence.rdates),
                }
            ),

            last_modified: Some(self.last_modified),
        }
    }
}









#[derive(Clone, Debug)]
pub struct EventSingle
{
    id: Uuid,

    /// If this is Some, it means this event is a single
    /// event that was originated from modifying the date/time
    /// of an event instance. That event instance does not exist
    /// anymore (i.e. won't be generated by `EventRecurring::generate_instances`)
    /// and this one "took its place".
    ///
    /// If this is None it just means this is a non-recurring event,
    /// without any relationship with any other event in the calendarold.
    ///
    /// For example:
    ///
    /// Imagine there's a recurrent event that starts at 2020-09-01 (Tue),
    /// happens weekly (every Tuesday), and has an ID of `abc`.
    /// Now imagine the user decided to move the instance of 2020-09-08
    /// one day ahead, making it happen on 2020-09-09.
    /// What happened "behind the scenes" is:
    /// 1. The date 2020-09-08 was added to the recurrent event's EXDATES property.
    /// 2. A (non-recurring) event was created at 2020-09-09, with the ID `cde`.
    /// 3. The parent_id of the `cde` event was set to `abc`.
    parent_id: Option<Uuid>,
    span: EventSpan,

    last_modified: NaiveDateTime,
}

impl EventSingle
{
    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_id(&self) -> Uuid { self.id }

    pub fn get_parent_id(&self) -> Option<Uuid> { self.parent_id }
}

impl ToPlain<EventPlain> for EventSingle
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: Some(self.id),
            parent_id: self.parent_id,

            start_date: Some(self.span.get_start_date()),
            end_date: Some(self.span.get_end_date()),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: None,

            last_modified: Some(self.last_modified),
        }
    }
}







#[derive(Clone, Debug)]
pub struct EventInstance
{
    parent_id: Uuid,
    span: EventSpan,
}

impl EventInstance
{
    pub fn get_span(&self) -> EventSpan { self.span }

    pub fn get_parent_id(&self) -> Uuid { self.parent_id }
}

impl ToPlain<EventPlain> for EventInstance
{
    fn into_plain(self) -> EventPlain
    {
        EventPlain {
            id: None,
            parent_id: Some(self.parent_id),

            start_date: Some(self.span.get_start_date()),
            end_date: Some(self.span.get_end_date()),
            start_time: self.span.get_start_time(),
            end_time: self.span.get_end_time(),

            recurrence: None,

            last_modified: None,
        }
    }
}


/// This is a serializable representation of an event
/// (single, recurrent or instance), it has two purposes:
///
/// 1) sending events to the client;
/// 2) receiving events from the client, validating them
/// and sending them to the database. Nothing else.
///
///
/// How to determine the type of the event:
///
/// - Single events don't have an rrule.
/// - Recurring events have an rrule value.
/// - Instance events don't have an id and have a parent id.
/// - Edited instance events (instance events with overridden
/// dates) have an id and a parent id.
///
/// If you want to create an EventPlain, call `to_plain`
/// on an `EventSingle`, `EventInstance` or `EventRecurring`.
///
/// All fields are optional to allow for PATCH methods.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct EventPlain
{
    pub id: Option<Uuid>,
    pub parent_id: Option<Uuid>,

    #[serde(default, with = "event_plain_serde::date_option")]
    #[schemars(with = "Option<NaiveDate>")]
    pub start_date: Option<NaiveDate>,

    #[serde(default, with = "event_plain_serde::time_option")]
    #[schemars(with = "Option<NaiveTime>")]
    pub start_time: Option<NaiveTime>,

    #[serde(default, with = "event_plain_serde::date_option")]
    #[schemars(with = "Option<NaiveDate>")]
    pub end_date: Option<NaiveDate>,

    #[serde(default, with = "event_plain_serde::time_option")]
    #[schemars(with = "Option<NaiveTime>")]
    pub end_time: Option<NaiveTime>,

    pub recurrence: Option<RecurrencePlain>,

    #[serde(default, with = "event_plain_serde::date_time_option")]
    #[schemars(with = "Option<NaiveDateTime>")]
    pub last_modified: Option<NaiveDateTime>,
}


/// Should only be used in conjunction with EventPlain.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct RecurrencePlain
{
    pub rrule: Option<String>,

    #[serde(default, with = "event_plain_serde::date_vec_option")]
    #[schemars(with = "Option<Vec<NaiveDate>>")]
    pub exdates: Option<Vec<NaiveDate>>,

    #[serde(default, with = "event_plain_serde::date_vec_option")]
    #[schemars(with = "Option<Vec<NaiveDate>>")]
    pub rdates: Option<Vec<NaiveDate>>
}

impl EventPlain
{
    /// Validate the event's data for a non-patch
    /// request. For example, if you're handling an
    /// insert request you want to make sure the event
    /// has at least a `start_date` and an `end_date` and
    /// also check some other integrity constraints.
    ///
    /// List of validation checks:
    ///
    /// - Checks if `start_date` and `end_date` are both set.
    /// - Checks if `end_time` is set if `start_time` is also set
    /// and vice-versa.
    /// - Checks if `rrule`, `exdates` and `rdates` are all set
    /// if `recurrence` is set.
    ///
    /// Returns `true` if the event is valid, `false` it it's not.
    pub fn validate_non_patch(&self) -> bool
    {
        if self.start_date.is_none() || self.end_date.is_none()
        {
            return false;
        }

        if self.start_time.is_some() != self.end_time.is_some()
        {
            return false;
        }

        if let Some(recurrence) = &self.recurrence
        {
            if recurrence.rrule.is_none()
                || recurrence.exdates.is_none()
                || recurrence.rdates.is_none()
            {
                return false;
            }
        }

        true
    }
}

pub trait ToPlain<T: Serialize + Deserialize<'static>>
{
    fn into_plain(self) -> T;
}

#[derive(Error, Debug)]
#[error("Could not transform the plain event into a recurrent or single event.")]
pub enum FromPlainError
{
    MissingField,
    InvalidSpan,
    RRuleParseError(RRuleParseError),
}


/// Provides serde functions for `Option<NaiveDate>`, `Option<NaiveTime>`
/// and `Option<Vec<NaiveDate>>`.
///
/// Dates are formatted like `YYYY-MM-DD`.
/// Times are formatted like `HH:MM:SS`.
mod event_plain_serde
{
    const DATE_FORMAT: &'static str = "%Y-%m-%d";
    const TIME_FORMAT: &'static str = "%H:%M";
    const DATE_TIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M";


    pub mod date_option
    {
        use chrono::{NaiveDate};
        use serde::{self, Deserialize, Serializer, Deserializer};

        use super::DATE_FORMAT;


        pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            match date
            {
                Some(date) => serializer.serialize_str(&format!("{}", date.format(DATE_FORMAT))),
                None => serializer.serialize_none()
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let string = Option::<String>::deserialize(deserializer)?;

            string
                .map(|s|
                    NaiveDate::parse_from_str(&s, DATE_FORMAT)
                        .map_err(serde::de::Error::custom)
                )
                .transpose()
        }
    }

    pub mod date_vec_option
    {
        use chrono::{NaiveDate};
        use serde::{self, Deserialize, Serializer, Deserializer};

        use serde::ser::SerializeSeq;
        use super::DATE_FORMAT;


        pub fn serialize<S>(dates: &Option<Vec<NaiveDate>>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            match dates
            {
                Some(dates) =>
                    {
                        let mut seq = serializer.serialize_seq(Some(dates.len()))?;

                        for date in dates
                        {
                            seq.serialize_element(&format!("{}", date.format(DATE_FORMAT)))?;
                        }

                        seq.end()
                    },
                None => serializer.serialize_none()
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<NaiveDate>>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let opt = Option::<Vec<String>>::deserialize(deserializer)?;

            opt
                .map(|vec|
                    vec
                        .into_iter()
                        .map(|x| NaiveDate::parse_from_str(&x, DATE_FORMAT))
                        .collect::<Result<Vec<NaiveDate>, _>>()
                        .map_err(serde::de::Error::custom)
                )
                .transpose()
        }
    }

    pub mod time_option
    {
        use chrono::{NaiveTime};
        use serde::{self, Deserialize, Serializer, Deserializer};

        use super::TIME_FORMAT;

        pub fn serialize<S>(date: &Option<NaiveTime>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            match date
            {
                Some(date) => serializer.serialize_str(&format!("{}", date.format(TIME_FORMAT))),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let string = Option::<String>::deserialize(deserializer)?;

            string
                .map(
                    |string| NaiveTime::parse_from_str(&string, TIME_FORMAT)
                        .map_err(serde::de::Error::custom)
                )
                .transpose()
        }
    }

    pub mod date_time_option
    {
        use chrono::{NaiveDateTime};
        use serde::{self, Deserialize, Serializer, Deserializer};

        use super::DATE_TIME_FORMAT;

        pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
        {
            match date
            {
                Some(date) => serializer.serialize_str(&format!("{}", date.format(DATE_TIME_FORMAT))),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
            where
                D: Deserializer<'de>,
        {
            let string = Option::<String>::deserialize(deserializer)?;

            string
                .map(
                    |string| NaiveDateTime::parse_from_str(&string, DATE_TIME_FORMAT)
                        .map_err(serde::de::Error::custom)
                )
                .transpose()
        }
    }
}
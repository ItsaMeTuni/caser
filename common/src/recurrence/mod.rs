use chrono::{NaiveDate, Month, Weekday};
use std::fmt::{Display, Formatter};

pub mod parser;
pub mod serde;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum RecurrenceFreq
{
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Display for RecurrenceFreq
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        let string = match self
        {
            RecurrenceFreq::Daily => "DAILY",
            RecurrenceFreq::Weekly => "WEEKLY",
            RecurrenceFreq::Monthly => "MONTHLY",
            RecurrenceFreq::Yearly => "YEARLY",
        };

        f.write_str(string)
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum RecurrenceLimit
{
    Indefinite,
    Date(NaiveDate),
    Count(u32),
}


/// An event's recurrence rule, this is used by `Event.generate_instances`
/// to figure out when event instances will happen.
/// This is basically a data structure to represent an
/// RRULE as defined in RFC 5545.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RecurrenceRule
{
    pub frequency: RecurrenceFreq,
    pub interval: i32,
    pub limit: RecurrenceLimit,


    pub by_month: Option<Vec<Month>>,
    pub by_week_no: Option<Vec<i32>>,
    pub by_year_day: Option<Vec<i32>>,
    pub by_month_day: Option<Vec<i32>>,
    pub by_day: Option<Vec<Weekday>>,
    pub by_set_pos: Option<Vec<i32>>,
}

impl RecurrenceRule
{
    /// Parses an RRULE string.
    pub fn new(rrule: &str) -> Result<Self, parser::RRuleParseError>
    {
        let rule = parser::parse(rrule)?;
        Ok(rule)
    }
}

impl Display for RecurrenceRule
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        let freq = format!("FREQ={}", self.frequency);

        let interval = if self.interval > 1
        {
            Some(format!("INTERVAL={}", self.interval))
        }
        else
        {
            None
        };

        let by_year_day = self.by_year_day.clone()
            .map(|x| format!("BYYEARDAY={}", vec_to_str(x)));

        let by_week_no = self.by_week_no.clone()
            .map(|x| format!("BYWEEKNO={}", vec_to_str(x)));

        let by_month_day = self.by_month_day.clone()
            .map(|x| format!("BYMONTHDAY={}", vec_to_str(x)));

        let by_set_pos = self.by_set_pos.clone()
            .map(|x| format!("BYSETPOS={}", vec_to_str(x)));

        let by_month = self.by_month.clone()
            .map(|x| x.iter()
                .map(|x| x.number_from_month().to_string())
                .collect::<Vec<String>>()
                .join(",")
            )
            .map(|x| format!("BYMONTH={}", x));

        let by_day = self.by_day.clone()
            .map(|x| x.iter()
                .map(|x| match x
                {
                    Weekday::Mon => "MO",
                    Weekday::Tue => "TU",
                    Weekday::Wed => "WE",
                    Weekday::Thu => "TH",
                    Weekday::Fri => "FR",
                    Weekday::Sat => "SA",
                    Weekday::Sun => "SU",
                })
                .collect::<Vec<&str>>()
                .join(",")
            )
            .map(|x| format!("BYDAY={}", x));


        let limit = match self.limit
        {
            RecurrenceLimit::Indefinite => None,
            RecurrenceLimit::Date(date) => Some(format!("UNTIL={}", date.format("%Y%m%d"))),
            RecurrenceLimit::Count(count) => Some(format!("COUNT={}", count)),
        };

        let string = vec![Some(freq), interval, by_year_day, by_day, by_week_no, by_month_day, by_set_pos, by_month, limit]
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<String>>()
            .join(";");

        f.write_str(&string)
    }
}

fn vec_to_str<T: Display>(vec: Vec<T>) -> String
{
    vec.iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(",")
}
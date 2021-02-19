//! This module does handles the event recurrence algorithm.

use chrono::{NaiveDate, Duration, Datelike, Weekday};
use self::helpers::NaiveDateHelpers;
use caser_common::recurrence::{RecurrenceRule, RecurrenceFreq, RecurrenceLimit};

mod helpers;

/// Wraps a `RecurrenceRule` along with a start date and makes sure
/// the rule is explicit.
///
/// An explicit rule means there are no values to be inferred.
/// E.g. picture a rule with FREQ=WEEKLY and BYDAY unset, that's an implicit
/// rule, since BYDAY has to be inferred from the event's start date. If, however,
/// the rule had BYDAY set, it would be an explicit rule, since no inference is
/// required.
///
/// The inference process is done at `RecurrenceRuleInstance::new`, so you can
/// feed it an implicit rule and it will make convert the rule to an explicit
/// one.
pub struct RecurrenceRuleInstance
{
    rule: RecurrenceRule,
    start_date: NaiveDate,
}

impl RecurrenceRuleInstance
{
    pub fn new(rule: &RecurrenceRule, start_date: NaiveDate) -> RecurrenceRuleInstance
    {
        RecurrenceRuleInstance {
            rule: Self::infer_stuff(rule.clone(), start_date),
            start_date,
        }
    }

    /// Returns a clone of this recurrence rule with
    /// inferred values if they're not already set.
    ///
    /// E.g.: if not already specified, BYDAY is inferred
    /// to be the same weekday as `starting_at` when
    /// FREQ=WEEKLY.
    fn infer_stuff(rule: RecurrenceRule, start_date: NaiveDate) -> RecurrenceRule
    {
        let mut new_by_day = None;
        let mut new_by_month_day = None;
        let mut new_by_year_day = None;

        // Infer BYDAY if recurrence is weekly
        if rule.frequency == RecurrenceFreq::Weekly && rule.by_day.is_none()
        {
            new_by_day = Some(vec![start_date.weekday()]);
        }

        // Infer BYMONTHDAY if recurrence is monthly
        if rule.frequency == RecurrenceFreq::Monthly && rule.by_month_day.is_none() && rule.by_day.is_none()
        {
            new_by_month_day = Some(vec![start_date.day() as i32]);
        }

        if rule.frequency == RecurrenceFreq::Yearly
        {
            // Infer BYMONTHDAY if BYMONTH is set
            if rule.by_month.is_some()
            {
                if rule.by_month_day.is_none()
                {
                    new_by_month_day = Some(vec![start_date.day() as i32]);
                }
            }
            // Infer BYDAY if BYWEEKNO is set
            else if rule.by_week_no.is_some()
            {
                if rule.by_day.is_none()
                {
                    new_by_day = Some(vec![start_date.weekday()]);
                }
            }
            // Infer BYYEARDAY if it's not set
            else if rule.by_year_day.is_none()
            {
                new_by_year_day = Some(vec![start_date.year_day() as i32]);
            }
        }

        RecurrenceRule {
            by_day: new_by_day,
            by_month_day: new_by_month_day,
            by_year_day: new_by_year_day,
            ..rule
        }
    }

    /// Calculate event instances based on this rule.
    ///
    ///
    /// Some rule properties might be inferred from `starting_at` if
    /// they're not present in the rule (e.g. if not already specified,
    /// BYDAY is inferred to be the same weekday as `starting_at`
    /// when FREQ=WEEKLY). You don't really have to worry about this
    /// unless you suspect there might be a bug with the inference
    /// algorithm. If you do, look at `infer_stuff`.
    pub fn calculate_instances(&self) -> RRuleInstances
    {
        RRuleInstances::new(self)
    }

    fn check_by_month(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_month) = &self.rule.by_month
        {
            by_month
                .iter()
                .find(|x| x.number_from_month() == date.month())
                .is_none()
        }
        else
        {
            true
        }
    }


    /// Check if `date` fits into the BYWEEKNO property of
    /// this rule.
    fn check_by_week_no(&self, _date: &NaiveDate) -> bool
    {
        if let Some(_by_week_no) = &self.rule.by_week_no
        {
            if self.rule.frequency != RecurrenceFreq::Yearly
            {
                panic!("by_week_no can only be used in a YEARLY recurrence.");
            }

            // TODO: implement this
            unimplemented!()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYYEARDAY property of
    /// this rule.
    fn check_by_year_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_year_day) = &self.rule.by_year_day
        {
            if matches!(self.rule.frequency, RecurrenceFreq::Daily | RecurrenceFreq::Weekly | RecurrenceFreq::Monthly)
            {
                panic!("by_year_day cannot be used in DAILY, WEEKLY, and MONTHLY recurrences.");
            }

            let year_day = date.year_day() as i32;
            by_year_day.iter().find(|x| **x == year_day).is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYMONTHDAY property of
    /// this rule.
    fn check_by_month_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_month_day) = &self.rule.by_month_day
        {
            if matches!(self.rule.frequency, RecurrenceFreq::Weekly)
            {
                panic!("by_month_day cannot be used in WEEKLY recurrences.");
            }

            let month_day = date.day() as i32;
            by_month_day.iter().find(|x| **x == month_day).is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYDAY property of
    /// this rule.
    fn check_by_day(&self, date: &NaiveDate) -> bool
    {
        if let Some(by_day) = &self.rule.by_day
        {
            by_day
                .iter()
                .find(|x| **x == date.weekday())
                .is_some()
        }
        else
        {
            true
        }
    }

    /// Check if `date` fits into the BYSETPOS property of
    /// this rule.
    fn check_by_set_pos(&self, _date: &NaiveDate) -> bool
    {
        if let Some(_by_set_pos) = &self.rule.by_set_pos
        {
            // TODO: implement this
            unimplemented!()
        }
        else
        {
            true
        }
    }

}

/// Calculates the recurrence instances for an event. I.e finds out the dates in which a recurring event
/// happens.
///
/// `starting_at` is the start date of the event. The date of the "original" event.
/// The function will only return dates between `from` and `to` (both inclusive).
///
///
/// ## How it works
///
/// Basically, we iterate through each date from `starting_at` until `to` and check if the
/// date matches the recurrence rule. If the date matches the rule and is between `from`
/// and `to` (both inclusive), we add it to the results vector.
///
/// ## A note on performance
/// This event is not very performant, it has an O(n) complexity where n is the number of days between
/// `starting_at` and `to`, so if `starting_at` is 2020-01-01 and `to` is 2021-01-01 the loop will execute 356
/// times. This doesn't seem so bad but if you have this function being called many times a second for events
/// a few years in the past this can quickly become a bottleneck. It works this way because I don't know any other
/// way to calculate the recurrence dates while taking into account all parameters as defined in RFC 5545. There
/// might be a better way to do this, but I don't know about it.
pub struct RRuleInstances<'rule>
{
    rule_instance: &'rule RecurrenceRuleInstance,
    instance_count: u32,
    last_instance_date: NaiveDate,
    current_date: NaiveDate,
}

impl<'rule> RRuleInstances<'rule>
{
    pub fn new(rule_instance: &RecurrenceRuleInstance) -> RRuleInstances
    {
        RRuleInstances {
            rule_instance,
            instance_count: 0,
            last_instance_date: rule_instance.start_date,
            current_date: rule_instance.start_date,
        }
    }
}

impl<'rule> Iterator for RRuleInstances<'rule>
{
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            let mut is_match = false;

            // Order matters here! This should be in the same order
            // as specified in RFC 5545
            let fits_into_rule =
                self.rule_instance.check_by_month(&self.current_date)
                    && self.rule_instance.check_by_week_no(&self.current_date)
                    && self.rule_instance.check_by_year_day(&self.current_date)
                    && self.rule_instance.check_by_month_day(&self.current_date)
                    && self.rule_instance.check_by_day(&self.current_date)
                    && self.rule_instance.check_by_set_pos(&self.current_date);

            match self.rule_instance.rule.limit
            {
                RecurrenceLimit::Indefinite => {},
                RecurrenceLimit::Date(date) =>
                    if self.current_date > date
                    {
                        break;
                    },
                RecurrenceLimit::Count(count) =>
                    if self.instance_count >= count
                    {
                        break;
                    },
            };

            if fits_into_rule
            {
                let freq_diff = match self.rule_instance.rule.frequency
                {
                    RecurrenceFreq::Daily => (self.current_date - self.last_instance_date).num_days(),
                    RecurrenceFreq::Weekly => calc_uniq_weeks_between(self.current_date, self.last_instance_date),
                    RecurrenceFreq::Monthly => {
                        if self.last_instance_date.month() > self.current_date.month()
                        {
                            (self.current_date.month() + 12 - self.last_instance_date.month()) as i64
                        }
                        else
                        {
                            (self.current_date.month() - self.last_instance_date.month()) as i64
                        }
                    },
                    RecurrenceFreq::Yearly => (self.current_date.year() - self.last_instance_date.year()) as i64,
                };

                if freq_diff >= self.rule_instance.rule.interval as i64 || freq_diff == 0
                {
                    self.instance_count += 1;

                    self.last_instance_date = self.current_date;

                    is_match = true;
                }
            }

            self.current_date += Duration::days(self.rule_instance.rule.interval as i64);

            if is_match
            {
                return Some(self.last_instance_date);
            }
        }

        None
    }
}

/// Calculates how many different weeks there are between
/// a and b. Positive if a > b, negative if a < b.
///
/// **IMPORTANT:** this does not calculate a week as exactly 7
/// days! If `a` is 2020-01-21 (Tue) and `b` is 2020-01-01 (Wed),
/// this function will return 4.
fn calc_uniq_weeks_between(a: NaiveDate, b: NaiveDate) -> i64
{
    let days_until_monday = a.iter_days().take_while(|x| x.weekday() != Weekday::Mon).count();

    let monday_date = a.iter_days().skip(days_until_monday).next().unwrap();

    (monday_date - b).num_weeks()
}

#[cfg(test)]
mod tests
{
    use super::*;
    use itertools::Itertools;

    const DEFAULT_RECURRENCE_RULE: RecurrenceRule = RecurrenceRule {
        frequency: RecurrenceFreq::Daily,
        interval: 1,
        limit: RecurrenceLimit::Indefinite,
        by_month: None,
        by_week_no: None,
        by_year_day: None,
        by_month_day: None,
        by_day: None,
        by_set_pos: None,
    };

    fn instances_between(rule: RecurrenceRuleInstance, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate>
    {
        rule.calculate_instances()
            .filter(|x| *x >= from)
            .take_while(|x| *x <= to)
            .collect_vec()
    }

    #[test]
    fn calc_recurrences_weekly_indefinite()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Indefinite,
            by_day: Some(vec![start_date.weekday()]),
            ..DEFAULT_RECURRENCE_RULE
        };

        let instance = RecurrenceRuleInstance::new(
            &rule,
            start_date
        );

        let result = instances_between(
            instance,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        );

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 22),
            NaiveDate::from_ymd(2020, 1, 29),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_weekly_w_date_limit()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Date(NaiveDate::from_ymd(2020, 1, 15)),
            by_day: Some(vec![start_date.weekday()]),
            ..DEFAULT_RECURRENCE_RULE
        };

        let instance = RecurrenceRuleInstance::new(
            &rule,
            start_date
        );

        let result = instances_between(
            instance,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        );

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_weekly_w_count_limit()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            limit: RecurrenceLimit::Count(4),
            by_day: Some(vec![start_date.weekday()]),
            ..DEFAULT_RECURRENCE_RULE
        };

        let instance = RecurrenceRuleInstance::new(
            &rule,
            start_date
        );

        let result = instances_between(
            instance,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        );

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 8),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 22),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn calc_recurrences_every_two_weeks()
    {
        let start_date = NaiveDate::from_ymd(2020, 1, 1);

        let rule = RecurrenceRule {
            frequency: RecurrenceFreq::Weekly,
            interval: 2,
            by_day: Some(vec![start_date.weekday()]),
            ..DEFAULT_RECURRENCE_RULE
        };

        let instance = RecurrenceRuleInstance::new(
            &rule,
            start_date
        );

        let result = instances_between(
            instance,
            NaiveDate::from_ymd(2020, 2, 1),
            NaiveDate::from_ymd(2020, 1, 1)
        );

        let expected = [
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2020, 1, 15),
            NaiveDate::from_ymd(2020, 1, 29),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn infer_by_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=WEEKLY").unwrap();
        let rule_instance = RecurrenceRuleInstance::new(&rule, start_date);

        assert_eq!(rule_instance.rule.by_day, Some(vec![Weekday::Sat]));
    }

    #[test]
    fn infer_by_month_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=MONTHLY").unwrap();
        let rule_instance = RecurrenceRuleInstance::new(&rule, start_date);

        assert_eq!(rule_instance.rule.by_month_day, Some(vec![26]));
    }

    #[test]
    fn yearly_infer_by_month_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY;BYMONTH=2").unwrap();
        let rule_instance = RecurrenceRuleInstance::new(&rule, start_date);

        assert_eq!(rule_instance.rule.by_month_day, Some(vec![26]));
    }

    #[test]
    fn yearly_infer_by_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY;BYWEEKNO=2,4,6").unwrap();
        let rule_instance = RecurrenceRuleInstance::new(&rule, start_date);

        assert_eq!(rule_instance.rule.by_day, Some(vec![Weekday::Sat]));
    }

    #[test]
    fn yearly_infer_by_year_day()
    {
        let start_date = NaiveDate::from_ymd(2020, 09, 26);
        let rule = RecurrenceRule::new("FREQ=YEARLY").unwrap();
        let rule_instance = RecurrenceRuleInstance::new(&rule, start_date);

        assert_eq!(rule_instance.rule.by_year_day, Some(vec![start_date.year_day() as i32]));
    }
}

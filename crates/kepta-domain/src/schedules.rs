use chrono::{Datelike, NaiveDate};
use kepta_types::schedules::FrequencyUnit;

/// Compute the next due date after completing a maintenance task.
pub fn next_due_after(last_completed: NaiveDate, unit: &FrequencyUnit, value: i32) -> NaiveDate {
    match unit {
        FrequencyUnit::Day => last_completed + chrono::Duration::days(value as i64),
        FrequencyUnit::Week => last_completed + chrono::Duration::weeks(value as i64),
        FrequencyUnit::Month => add_months(last_completed, value),
        FrequencyUnit::Year => add_months(last_completed, value * 12),
    }
}

fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total_months = date.month() as i32 - 1 + months;
    let year = date.year() + total_months / 12;
    let month = (total_months % 12 + 1) as u32;
    let day = date
        .day()
        .min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .and_then(|d| d.pred_opt())
    .map(|d| d.day())
    .unwrap_or(28)
}

/// How many days until (or since) a schedule is due.
/// Negative = overdue.
pub fn days_until_due(next_due: NaiveDate, today: NaiveDate) -> i64 {
    (next_due - today).num_days()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // next_due_after

    #[test]
    fn daily_adds_exact_days() {
        let result = next_due_after(date(2024, 1, 1), &FrequencyUnit::Day, 30);
        assert_eq!(result, date(2024, 1, 31));
    }

    #[test]
    fn weekly_adds_exact_weeks() {
        let result = next_due_after(date(2024, 1, 1), &FrequencyUnit::Week, 2);
        assert_eq!(result, date(2024, 1, 15));
    }

    #[test]
    fn monthly_same_day_next_month() {
        let result = next_due_after(date(2024, 1, 15), &FrequencyUnit::Month, 1);
        assert_eq!(result, date(2024, 2, 15));
    }

    #[test]
    fn monthly_clamps_to_last_day_of_short_month() {
        // Jan 31 + 1 month → Feb 29 (2024 is a leap year)
        let result = next_due_after(date(2024, 1, 31), &FrequencyUnit::Month, 1);
        assert_eq!(result, date(2024, 2, 29));
    }

    #[test]
    fn monthly_clamps_feb_non_leap_year() {
        let result = next_due_after(date(2023, 1, 31), &FrequencyUnit::Month, 1);
        assert_eq!(result, date(2023, 2, 28));
    }

    #[test]
    fn monthly_crosses_year_boundary() {
        let result = next_due_after(date(2024, 11, 15), &FrequencyUnit::Month, 3);
        assert_eq!(result, date(2025, 2, 15));
    }

    #[test]
    fn yearly_same_day_next_year() {
        let result = next_due_after(date(2024, 3, 1), &FrequencyUnit::Year, 1);
        assert_eq!(result, date(2025, 3, 1));
    }

    #[test]
    fn multi_year_interval() {
        let result = next_due_after(date(2024, 6, 1), &FrequencyUnit::Year, 5);
        assert_eq!(result, date(2029, 6, 1));
    }

    // days_until_due

    #[test]
    fn due_in_future_is_positive() {
        assert_eq!(days_until_due(date(2024, 6, 10), date(2024, 6, 7)), 3);
    }

    #[test]
    fn due_today_is_zero() {
        assert_eq!(days_until_due(date(2024, 6, 7), date(2024, 6, 7)), 0);
    }

    #[test]
    fn overdue_is_negative() {
        assert_eq!(days_until_due(date(2024, 6, 1), date(2024, 6, 7)), -6);
    }
}

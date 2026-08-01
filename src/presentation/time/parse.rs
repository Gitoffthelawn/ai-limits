use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

use super::TimeContext;

pub(super) fn parse_instant_reference(value: &str) -> Option<DateTime<Local>> {
    parse_iso_or_unix(value)
}

pub(super) fn parse_to_local(value: &str, context: &TimeContext) -> Option<DateTime<Local>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_iso_or_unix(trimmed) {
        return Some(parsed);
    }

    let (body, timezone) = split_timezone_suffix(trimmed);
    if let Some(parsed) = parse_source_specific(body, timezone, context) {
        return Some(parsed);
    }

    None
}

pub(super) fn parse_iso_or_unix(value: &str) -> Option<DateTime<Local>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Local));
    }
    for format in [
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%S%.3fZ",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(Utc.from_utc_datetime(&parsed).with_timezone(&Local));
        }
    }
    if value.chars().all(|character| character.is_ascii_digit()) {
        let seconds = value.parse::<i64>().ok()?;
        return Utc
            .timestamp_opt(seconds, 0)
            .single()
            .map(|parsed| parsed.with_timezone(&Local));
    }

    None
}

fn split_timezone_suffix(value: &str) -> (&str, Option<Tz>) {
    let Some(open) = value.rfind('(') else {
        return (value, None);
    };
    if !value.ends_with(')') {
        return (value, None);
    }

    let name = value[open + 1..value.len() - 1].trim();
    let body = value[..open].trim();
    (body, name.parse::<Tz>().ok())
}

fn parse_source_specific(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    if let Some(parsed) = parse_on_date_format(body, timezone, context) {
        return Some(parsed);
    }
    if let Some(parsed) = parse_month_day_at_time(body, timezone, context) {
        return Some(parsed);
    }
    if let Some(parsed) = parse_time_only(body, timezone, context) {
        return Some(parsed);
    }

    None
}

fn parse_on_date_format(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let (time_part, date_part) = body.split_once(" on ")?;
    let date = parse_day_month(date_part.trim(), context.reference.year())?;
    let time = parse_clock_time(time_part.trim())?;
    assemble_local_datetime(date, time, timezone, context, DateRollPolicy::YearIfPast)
}

fn parse_month_day_at_time(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let (date_part, time_part) = body.split_once(" at ")?;
    let date = parse_month_day(date_part.trim(), context.reference.year())?;
    let time = parse_clock_time(time_part.trim())?;
    assemble_local_datetime(date, time, timezone, context, DateRollPolicy::YearIfPast)
}

fn parse_time_only(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let time = parse_clock_time(body)?;
    assemble_local_datetime(
        context.reference.date_naive(),
        time,
        timezone,
        context,
        DateRollPolicy::DayIfPast,
    )
}

enum DateRollPolicy {
    DayIfPast,
    YearIfPast,
}

fn assemble_local_datetime(
    mut date: NaiveDate,
    time: NaiveTime,
    timezone: Option<Tz>,
    context: &TimeContext,
    roll: DateRollPolicy,
) -> Option<DateTime<Local>> {
    let mut local = localize_naive(date, time, timezone)?;

    match roll {
        DateRollPolicy::DayIfPast if local <= context.reference => {
            date = date.succ_opt()?;
            local = localize_naive(date, time, timezone)?;
        }
        DateRollPolicy::YearIfPast if local <= context.reference => {
            date = NaiveDate::from_ymd_opt(date.year() + 1, date.month(), date.day())?;
            local = localize_naive(date, time, timezone)?;
        }
        _ => {}
    }

    Some(local)
}

fn localize_naive(
    date: NaiveDate,
    time: NaiveTime,
    timezone: Option<Tz>,
) -> Option<DateTime<Local>> {
    let naive = NaiveDateTime::new(date, time);
    if let Some(timezone) = timezone {
        return timezone
            .from_local_datetime(&naive)
            .single()
            .map(|parsed| parsed.with_timezone(&Local));
    }

    Local
        .from_local_datetime(&naive)
        .single()
        .map(|parsed| parsed.with_timezone(&Local))
}

fn parse_day_month(value: &str, year: i32) -> Option<NaiveDate> {
    let mut parts = value.split_whitespace();
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = parse_month_name(parts.next()?)?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_month_day(value: &str, year: i32) -> Option<NaiveDate> {
    let mut parts = value.split_whitespace();
    let month = parse_month_name(parts.next()?)?;
    let day = parts.next()?.parse::<u32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_month_name(value: &str) -> Option<u32> {
    let normalized = value.trim_end_matches('.').to_ascii_lowercase();
    match normalized.as_str() {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "sept" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn parse_clock_time(value: &str) -> Option<NaiveTime> {
    let trimmed = value.trim();
    if let Ok(parsed) = NaiveTime::parse_from_str(trimmed, "%H:%M") {
        return Some(parsed);
    }

    let lower = trimmed.to_ascii_lowercase();
    let (body, pm, is_12h) = if let Some(body) = lower.strip_suffix("am") {
        (body, false, true)
    } else if let Some(body) = lower.strip_suffix("pm") {
        (body, true, true)
    } else {
        (lower.as_str(), false, false)
    };

    let body = body.trim();
    let (hour, minute) = match body.split_once(':') {
        Some((hour, minute)) => (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?),
        None => (body.parse::<u32>().ok()?, 0),
    };

    let hour = if is_12h {
        if pm {
            if hour == 12 {
                12
            } else {
                hour + 12
            }
        } else if hour == 12 {
            0
        } else {
            hour
        }
    } else {
        hour
    };

    NaiveTime::from_hms_opt(hour, minute, 0)
}

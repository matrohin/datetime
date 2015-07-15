use local::{LocalDate, LocalTime, LocalDateTime, Month};
use zoned::*;

use regex::Regex;

/// Splits Date String, Time String
///
/// for further parsing by `parse_iso_8601_date` and `parse_iso_8601_time`.
pub fn split_iso_8601(string:&str) -> Option<(&str, &str)> {
    let split = Regex::new(r"^([^T]*)T?(.*)$").unwrap();
    if split.is_match(&string) {
        let caps = split.captures(&string).unwrap();
        if caps.len() > 1 {
            return Some( (caps.at(1).unwrap().into(), caps.at(2).unwrap().into()) );
        }
    }
    None
}

/// Parses a ISO 8601 a string into LocalDateTime Object.
pub fn parse_iso_8601(string:&str) -> Option<LocalDateTime> {
    let (date_string, time_string) = split_iso_8601(string).unwrap();
    match (parse_iso_8601_date(&date_string), parse_iso_8601_time(&time_string)) {
        (Some(date),Some(time)) => return Some(LocalDateTime::from_date_time(date,time)),
        _ => None
    }
}


/// Parses ISO 8601 Date a string into a LocalDate Object.
///
/// Used by `LocalDate::parse()`
pub fn parse_iso_8601_date(string:&str) -> Option<LocalDate> {
    let week = Regex::new(r##"(?x)^
        (\d{4})   # year
        -W(\d{2}) # number of week
        -(\d{1})  # day in week (1..7)//}
        $"##).unwrap();
    let ymd  = Regex::new(r##"(?x)^
        (\d{4})   # year
        -?(\d{2}) # month
        -?(\d{2}) # day
        $"##).unwrap();

    if ymd.is_match(&string) {
        ymd.captures(string).map(|caps|
        LocalDate::new(
            caps.at(1).unwrap().parse().unwrap(), // year
            Month::from_one(caps.at(2).unwrap().parse().unwrap()), // month
            caps.at(3).unwrap().parse().unwrap(), // day
            ).unwrap())
    }
    else if week.is_match(&string) {
        week.captures(string).map(|caps|
        LocalDate::from_weekday(
            caps.at(1).unwrap().parse().unwrap(), // year
            caps.at(2).unwrap().parse().unwrap(), // week
            caps.at(3).unwrap().parse().unwrap()  // weekday
            ).unwrap())
    }
    else { None }
}

/// Parses ISO 8601 a string into a ZonedDateTime Object.
///
/// Used by `ZonedDateTime::parse()`
pub fn parse_iso_8601_zoned(string:&str) -> Option<(LocalDateTime, TimeZone)> {
    let (date_string, time_string) = split_iso_8601(string).unwrap();
    match (parse_iso_8601_date(&date_string),parse_iso_8601_tuple(&time_string)){
        (Some(date), Some((hour, minute, second, millisecond, zh, zm, z)) ) => {
            if let Some(time) = LocalTime::hms_ms(hour, minute, second, millisecond as i16){
                let time_zone = if z == "Z" {
                    TimeZone::UTC
                } else {
                    TimeZone::of_hours_and_minutes(zh,zm)
                };

                Some(( LocalDateTime::from_date_time(date,time), time_zone))
            } else {None}
        },
        (Some(date), None) => {
            if let Some(time) = LocalTime::hms(0,0,0){
                Some(( LocalDateTime::from_date_time(date,time), TimeZone::UTC))
            } else {None}
        }
        _ => None
    }
}

/// Parses ISO 8601 a string into a LocalTime Object.
///
/// Used by `LocalTime::parse()`
pub fn parse_iso_8601_time(string:&str) -> Option<LocalTime> {
    if string.is_empty() {
        return Some(LocalTime::hms(0,0,0).unwrap());
    }
    if let Some((hour, minute, second, millisecond, _zh, _zm, _z)) = parse_iso_8601_tuple(string){
        return LocalTime::hms_ms(hour, minute, second, millisecond as i16);
    }
    None
}

// implementation detail
fn parse_iso_8601_tuple(string:&str) -> Option<(i8,i8,i8,i32,i8,i8,&str)> {
    let exp = Regex::new(r##"(?x) ^
        (\d{2}) :?     # hour
        (\d{2})? :?    # minute

        (?:
            (\d{2})         # second
            \.?
            ((?:\d{1,9}))?  # millisecond
        )?

        (?:                 # time zone offset:
            (Z) |           # or just Z for UTC
            ([+-]\d\d)? :?  # hour and
            (\d\d)?         # minute,
        )?
    $"##).ok().expect("Regex Broken");

    if exp.is_match(&string) {
        let tup = exp.captures(string).map(|caps|
               (
                caps.at(1).unwrap_or("00").parse::<i8>().unwrap(), // HH
                caps.at(2).unwrap_or("00").parse::<i8>().unwrap(), // MM
                caps.at(3).unwrap_or("00").parse::<i8>().unwrap(), // SS
                caps.at(4).unwrap_or("000").parse::<i32>().unwrap(), // MS
                caps.at(6).unwrap_or("+00").trim_matches('+').parse::<i8>().unwrap(), // ZH
                caps.at(7).unwrap_or("00").parse::<i8>().unwrap(), // ZM
                caps.at(5).unwrap_or("_"), // "Z"
                )).unwrap();

        //TODO: check this with the rfc3339 standard
        //if tup.3 > 0 && &format!("{}", tup.3).len() %3 != 0{ return None}
        return Some(tup);

    }
    None
}


#[cfg(test)]
mod test {
    pub use super::parse_iso_8601_date;
    pub use local::{LocalDate, Month};

    #[test]
    fn date() {
        let date = parse_iso_8601_date("1985-04-12");
        assert_eq!(date, LocalDate::new(1985, Month::April, 12));
    }

    #[test]
    fn fail() {
        let date = parse_iso_8601_date("");
        assert_eq!(date, None);
    }
}

// 2014-12-25
// Combined date and time in UTC:   2014-12-25T02:56:40+00:00, 2014-12-25T02:56:40Z
// Week:    2014-W52
// Date with week number:   2014-W52-4
// Ordinal date:    2014-359

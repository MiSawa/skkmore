use time::{format_description::FormatItem, macros::format_description, Duration, OffsetDateTime};

fn time_candidates(time: OffsetDateTime) -> Vec<String> {
    const FORMATS: [&[FormatItem<'static>]; 2] = [
        format_description!("[year]/[month]/[day]"),
        format_description!("[year]-[month]-[day]"),
    ];
    FORMATS.iter().flat_map(|f| time.format(f).ok()).collect()
}

fn time_conversion(offset_from_now: Duration) -> Vec<String> {
    // NB: Has to be in a single-threaded environment, or under a multithread-safe OS to be able to
    // get local offset.
    if let Some(time) = OffsetDateTime::now_local()
        .ok()
        .and_then(|t| t.checked_add(offset_from_now))
    {
        time_candidates(time)
    } else {
        vec![]
    }
}

pub fn convert(input: &str) -> Vec<String> {
    match input {
        "おととい" => time_conversion(-Duration::DAY * 2),
        "きのう" => time_conversion(-Duration::DAY),
        "きょう" => time_conversion(Duration::ZERO),
        "あした" | "あす" => time_conversion(Duration::DAY),
        "あさって" => time_conversion(Duration::DAY * 2),
        _ => vec![],
    }
}

#[cfg(test)]
mod test {
    use time::macros::datetime;

    use super::*;
    #[test]
    fn test_time_candidates() {
        assert_eq!(
            time_candidates(datetime!(2022-02-23 04:05:06 +9:00)),
            vec!["2022/02/23".to_string(), "2022-02-23".to_string()]
        );
    }
}

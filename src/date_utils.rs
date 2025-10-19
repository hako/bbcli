use chrono::{DateTime, Utc};

/// Humanizes a date string from "YYYY-MM-DD HH:MM:SS" format to relative time
/// Returns strings like "just now", "5 minutes ago", "2 hours ago", "yesterday", etc.
pub fn humanize_time(date_str: &str) -> String {
    // Try to parse the date string (format: "YYYY-MM-DD HH:MM:SS")
    let parsed = DateTime::parse_from_str(&format!("{} +0000", date_str), "%Y-%m-%d %H:%M:%S %z");

    let date_time = match parsed {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => {
            // If parsing fails, return the original string
            return date_str.to_string();
        }
    };

    let now = Utc::now();
    let duration = now.signed_duration_since(date_time);

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if seconds < 0 {
        // Future date
        return "just now".to_string();
    } else if seconds < 60 {
        return "just now".to_string();
    } else if minutes < 60 {
        if minutes == 1 {
            return "1 minute ago".to_string();
        } else {
            return format!("{} minutes ago", minutes);
        }
    } else if hours < 24 {
        if hours == 1 {
            return "1 hour ago".to_string();
        } else {
            return format!("{} hours ago", hours);
        }
    } else if days == 1 {
        return "yesterday".to_string();
    } else if days < 7 {
        return format!("{} days ago", days);
    } else if days < 30 {
        let weeks = days / 7;
        if weeks == 1 {
            return "1 week ago".to_string();
        } else {
            return format!("{} weeks ago", weeks);
        }
    } else if days < 365 {
        let months = days / 30;
        if months == 1 {
            return "1 month ago".to_string();
        } else {
            return format!("{} months ago", months);
        }
    } else {
        let years = days / 365;
        if years == 1 {
            return "1 year ago".to_string();
        } else {
            return format!("{} years ago", years);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_humanize_recent() {
        let now = Utc::now();

        // Just now
        let recent = (now - Duration::seconds(30)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(humanize_time(&recent), "just now");

        // 5 minutes ago
        let mins = (now - Duration::minutes(5)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(humanize_time(&mins), "5 minutes ago");
    }

    #[test]
    fn test_humanize_hours() {
        let now = Utc::now();
        let hours = (now - Duration::hours(3)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(humanize_time(&hours), "3 hours ago");
    }

    #[test]
    fn test_humanize_days() {
        let now = Utc::now();
        let yesterday = (now - Duration::days(1)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(humanize_time(&yesterday), "yesterday");

        let days = (now - Duration::days(5)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(humanize_time(&days), "5 days ago");
    }

    #[test]
    fn test_invalid_date() {
        let invalid = "invalid date";
        assert_eq!(humanize_time(invalid), "invalid date");
    }
}

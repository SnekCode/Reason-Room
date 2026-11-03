use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Refresh when application code, workspace dependencies, or their resolution
    // changes. An unchanged artifact keeps the timestamp of its actual build.
    for path in [
        "build.rs",
        "Cargo.toml",
        "src",
        "../canvas",
        "../workspace-session",
        "../../Cargo.toml",
        "../../Cargo.lock",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the build clock must be after the Unix epoch")
        .as_secs();
    println!(
        "cargo:rustc-env=REASON_ROOM_BUILD_UTC={}",
        utc_timestamp(seconds)
    );
}

fn utc_timestamp(seconds: u64) -> String {
    let mut days = seconds / 86_400;
    let mut year = 1970;
    let leap_year = |year| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    loop {
        let year_days = if leap_year(year) { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }
    let months = [
        31,
        if leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for month_days in months {
        if days < month_days {
            break;
        }
        days -= month_days;
        month += 1;
    }
    let time = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{:02} {:02}:{:02}:{:02} UTC",
        days + 1,
        time / 3600,
        time / 60 % 60,
        time % 60
    )
}

#[cfg(test)]
mod tests {
    use super::utc_timestamp;

    #[test]
    fn formats_utc_without_host_timezone_or_external_commands() {
        assert_eq!(utc_timestamp(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(utc_timestamp(86_399), "1970-01-01 23:59:59 UTC");
        assert_eq!(utc_timestamp(951_782_400), "2000-02-29 00:00:00 UTC");
        assert_eq!(utc_timestamp(4_107_542_400), "2100-03-01 00:00:00 UTC");
        assert_eq!(utc_timestamp(1_793_577_600), "2026-11-02 00:00:00 UTC");
    }
}

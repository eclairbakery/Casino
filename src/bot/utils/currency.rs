pub fn parse_to_minor(input: &str) -> Result<i64, ()> {
    let mut parts = input.trim().split('.');

    let major: i64 = parts.next().ok_or(())?.parse().or(Err(()))?;
    let frac = parts.next().unwrap_or("");

    if parts.next().is_some() || frac.len() > 2 {
        return Err(());
    }

    let minor = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().or(Err(()))? * 10,
        2 => frac.parse::<i64>().or(Err(()))?,
        _ => unreachable!(),
    };

    let cents = if major >= 0 { minor } else { -minor };

    major
        .checked_mul(100)
        .and_then(|v| v.checked_add(cents))
        .ok_or(())
}

pub fn format_minor(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };

    let abs = value.abs();
    let major = abs / 100;
    let minor = abs % 100;

    format!("{sign}{major}.{minor:02}zł")
}

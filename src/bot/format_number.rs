pub fn format_number(n: i64) -> String {
    format!("{:.2}", n as f64 / 100.0)
}

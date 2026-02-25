use serenity::all::Color;

pub enum LogType {
	Success,
	Info,
	Tip,
	Warn,
	Error,
}

pub struct LogConfig {
	pub emoji: String,
	pub color: Color
}

impl LogConfig {
	pub fn new(emoji: String, color: Color) -> Self {
		Self { emoji, color }
	}
}
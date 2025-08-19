#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
  None,
  Verbose,
}

impl Default for LogLevel {
  fn default() -> Self {
    LogLevel::None
  }
}

#[derive(Debug)]
pub struct Logger {
  level: LogLevel,
}

impl Logger {
  pub fn new(level: LogLevel) -> Self {
    Logger { level }
  }

  pub fn log_step(&self, step: &str) {
    if self.level == LogLevel::Verbose {
      println!("[STEP] {}", step);
    }
  }

  pub fn log_output(&self, output: &str) {
    if self.level == LogLevel::Verbose {
      println!("[OUTPUT] {}", output);
    }
  }
}

impl Default for Logger {
  fn default() -> Self {
    Logger::new(LogLevel::None)
  }
}

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

  /// Execute a future with automatic logging
  pub async fn exec<T, F, Fut>(&self, step_name: &str, future: F) -> T
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
  {
    self.log_step(step_name);
    future().await
  }

  /// Execute a future with automatic logging and result formatting
  pub async fn exec_with_output<T, F, Fut, G>(&self, step_name: &str, future: F, format_fn: G) -> T
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
    G: FnOnce(&T) -> String,
    T: Clone,
  {
    self.log_step(step_name);
    let result = future().await;
    self.log_output(&format_fn(&result));
    result
  }

  /// Execute a Result-returning future with automatic logging
  pub async fn exec_result<T, E, F, Fut>(&self, step_name: &str, future: F) -> Result<T, E>
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
  {
    self.log_step(step_name);
    future().await
  }

  /// Execute a Result-returning future with automatic logging and result formatting
  pub async fn exec_result_with_output<T, E, F, Fut, G>(&self, step_name: &str, future: F, format_fn: G) -> Result<T, E>
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    G: FnOnce(&T) -> String,
  {
    self.log_step(step_name);
    let result = future().await;
    if let Ok(ref value) = result {
      self.log_output(&format_fn(value));
    }
    result
  }

  /// Execute a synchronous Result-returning operation with automatic logging
  pub fn exec_sync_result<T, E, F>(&self, step_name: &str, operation: F) -> Result<T, E>
  where
    F: FnOnce() -> Result<T, E>,
  {
    self.log_step(step_name);
    operation()
  }

  /// Execute a synchronous Result-returning operation with automatic logging and result formatting
  pub fn exec_sync_result_with_output<T, E, F, G>(&self, step_name: &str, operation: F, format_fn: G) -> Result<T, E>
  where
    F: FnOnce() -> Result<T, E>,
    G: FnOnce(&T) -> String,
  {
    self.log_step(step_name);
    let result = operation();
    if let Ok(ref value) = result {
      self.log_output(&format_fn(value));
    }
    result
  }
}

impl Default for Logger {
  fn default() -> Self {
    Logger::new(LogLevel::None)
  }
}

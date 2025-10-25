use crate::core::SpeedTestResult;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};

/// Progress bar for speed testing
pub struct SpeedTestProgress {
    bar: Arc<Mutex<ProgressBar>>,
}

impl SpeedTestProgress {
    /// Create a new progress bar
    pub fn new(total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        // bar.set_message("Initializing...");

        Self {
            bar: Arc::new(Mutex::new(bar))
        }
    }

    /// Update progress with a new result
    pub fn update(&self, result: &SpeedTestResult) {
        if let Ok(bar) = self.bar.lock() {
            bar.inc(1);

            let status = if result.is_successful() {
                format!("✓ {} ({})", result.proxy_name, result.format_latency())
            } else {
                format!("✗ {} (Failed)", result.proxy_name)
            };

            bar.set_message(status);
        }
    }

    /// Set a custom message
    pub fn set_message(&self, msg: &str) {
        if let Ok(bar) = self.bar.lock() {
            bar.set_message(msg.to_string());
        }
    }

    /// Finish the progress bar
    pub fn finish_with_message(&self, msg: &str) {
        if let Ok(bar) = self.bar.lock() {
            bar.finish_with_message(msg.to_string());
        }
    }

    /// Clear the progress bar (not available in all versions)
    pub fn clear(&self) {
        // Clear is not available in all versions of indicatif
        // if let Ok(mut bar) = self.bar.lock() {
        //     bar.clear();
        // }
    }
}

impl Drop for SpeedTestProgress {
    fn drop(&mut self) {
        if let Ok(bar) = self.bar.lock() {
            bar.finish_and_clear();
        }
    }
}

impl Clone for SpeedTestProgress {
    fn clone(&self) -> Self {
        Self {
            bar: Arc::clone(&self.bar)
        }
    }
}

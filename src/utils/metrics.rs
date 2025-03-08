use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A simple histogram for tracking latency distributions
#[derive(Default, Debug, Clone)]
pub struct Histogram {
    /// Counts for each bucket (in microseconds)
    counts: HashMap<u64, u64>,
    /// Total number of observations
    count: u64,
    /// Sum of all observed values
    sum: u64,
    /// Minimum observed value
    min: Option<u64>,
    /// Maximum observed value
    max: Option<u64>,
}

impl Histogram {
    /// Creates a new empty histogram
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            count: 0,
            sum: 0,
            min: None,
            max: None,
        }
    }
    
    /// Records a new observation
    pub fn observe(&mut self, value: u64) {
        // Round to the nearest bucket
        let bucket = self.bucket_for(value);
        
        // Update counts
        *self.counts.entry(bucket).or_insert(0) += 1;
        self.count += 1;
        self.sum += value;
        
        // Update min/max
        self.min = match self.min {
            None => Some(value),
            Some(min) => Some(min.min(value)),
        };
        
        self.max = match self.max {
            None => Some(value),
            Some(max) => Some(max.max(value)),
        };
    }
    
    /// Gets the bucket for a value
    fn bucket_for(&self, value: u64) -> u64 {
        // Simple bucketing strategy: round to nearest power of 2
        if value == 0 {
            return 0;
        }
        
        // Find the highest bit position
        let highest_bit = 63 - value.leading_zeros();
        
        // Calculate the bucket
        1u64 << highest_bit
    }
    
    /// Returns the count of observations
    pub fn count(&self) -> u64 {
        self.count
    }
    
    /// Returns the sum of all observations
    pub fn sum(&self) -> u64 {
        self.sum
    }
    
    /// Returns the average of all observations
    pub fn average(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum as f64 / self.count as f64)
        } else {
            None
        }
    }
    
    /// Returns the minimum observed value
    pub fn min(&self) -> Option<u64> {
        self.min
    }
    
    /// Returns the maximum observed value
    pub fn max(&self) -> Option<u64> {
        self.max
    }
    
    /// Returns the median (50th percentile)
    pub fn median(&self) -> Option<u64> {
        self.percentile(50.0)
    }
    
    /// Returns the value at the given percentile
    pub fn percentile(&self, percentile: f64) -> Option<u64> {
        if self.count == 0 {
            return None;
        }
        
        // Validate percentile
        if !(0.0..=100.0).contains(&percentile) {
            return None;
        }
        
        // Calculate the rank
        let rank = (percentile / 100.0 * self.count as f64).ceil() as u64;
        
        // Sort buckets
        let mut sorted_buckets: Vec<_> = self.counts.iter().collect();
        sorted_buckets.sort_by_key(|&(bucket, _)| *bucket);
        
        // Find the bucket containing the rank
        let mut cumulative = 0;
        for (bucket, count) in sorted_buckets {
            cumulative += count;
            if cumulative >= rank {
                return Some(*bucket);
            }
        }
        
        // This should not happen if count > 0
        None
    }
    
    /// Merges another histogram into this one
    pub fn merge(&mut self, other: &Histogram) {
        for (&bucket, &count) in &other.counts {
            *self.counts.entry(bucket).or_insert(0) += count;
        }
        
        self.count += other.count;
        self.sum += other.sum;
        
        self.min = match (self.min, other.min) {
            (None, None) => None,
            (Some(min), None) => Some(min),
            (None, Some(min)) => Some(min),
            (Some(min1), Some(min2)) => Some(min1.min(min2)),
        };
        
        self.max = match (self.max, other.max) {
            (None, None) => None,
            (Some(max), None) => Some(max),
            (None, Some(max)) => Some(max),
            (Some(max1), Some(max2)) => Some(max1.max(max2)),
        };
    }
    
    /// Returns a string representation of the histogram
    pub fn summary(&self) -> String {
        if self.count == 0 {
            return "No data".to_string();
        }
        
        format!(
            "count: {}, avg: {:.2} µs, min: {} µs, p50: {} µs, p95: {} µs, p99: {} µs, max: {} µs",
            self.count,
            self.average().unwrap_or(0.0),
            self.min.unwrap_or(0),
            self.percentile(50.0).unwrap_or(0),
            self.percentile(95.0).unwrap_or(0),
            self.percentile(99.0).unwrap_or(0),
            self.max.unwrap_or(0)
        )
    }
}

/// A simple metric for measuring execution times
pub struct Timer {
    /// Name of the timer
    name: String,
    /// Start time
    start: Instant,
    /// Histogram to record observations
    histogram: Arc<Mutex<Histogram>>,
}

impl Timer {
    /// Creates a new timer
    pub fn new(name: &str, histogram: Arc<Mutex<Histogram>>) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
            histogram,
        }
    }
    
    /// Stops the timer and records the elapsed time
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        
        // Record in microseconds
        if let Ok(mut histogram) = self.histogram.lock() {
            histogram.observe(elapsed.as_micros() as u64);
        }
        
        elapsed
    }
}

/// A registry of metrics
pub struct MetricsRegistry {
    /// Histograms by name
    histograms: HashMap<String, Arc<Mutex<Histogram>>>,
}

impl MetricsRegistry {
    /// Creates a new empty registry
    pub fn new() -> Self {
        Self {
            histograms: HashMap::new(),
        }
    }
    
    /// Gets or creates a histogram
    pub fn histogram(&mut self, name: &str) -> Arc<Mutex<Histogram>> {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(Histogram::new())))
            .clone()
    }
    
    /// Creates a new timer
    pub fn timer(&mut self, name: &str) -> Timer {
        let histogram = self.histogram(name);
        Timer::new(name, histogram)
    }
    
    /// Returns a summary of all metrics
    pub fn summary(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        
        for (name, histogram) in &self.histograms {
            if let Ok(histogram) = histogram.lock() {
                result.insert(name.clone(), histogram.summary());
            }
        }
        
        result
    }
    
    /// Resets all metrics
    pub fn reset(&mut self) {
        for (_, histogram) in &self.histograms {
            if let Ok(mut histogram) = histogram.lock() {
                *histogram = Histogram::new();
            }
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_histogram() {
        let mut hist = Histogram::new();
        
        // Empty histogram
        assert_eq!(hist.count(), 0);
        assert_eq!(hist.min(), None);
        assert_eq!(hist.max(), None);
        assert_eq!(hist.average(), None);
        
        // Add some values
        hist.observe(10);
        hist.observe(20);
        hist.observe(30);
        
        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 60);
        assert_eq!(hist.min(), Some(10));
        assert_eq!(hist.max(), Some(30));
        assert_eq!(hist.average(), Some(20.0));
    }
    
    #[test]
    fn test_timer() {
        let registry = Arc::new(Mutex::new(MetricsRegistry::new()));
        
        {
            let mut registry = registry.lock().unwrap();
            let timer = registry.timer("test_timer");
            sleep(Duration::from_millis(5));
            let elapsed = timer.stop();
            
            assert!(elapsed.as_millis() >= 5);
            
            // Check that the histogram was updated
            let histogram = registry.histogram("test_timer");
            let hist = histogram.lock().unwrap();
            assert_eq!(hist.count(), 1);
            assert!(hist.min().unwrap() >= 5000); // at least 5000 microseconds
        }
    }
    
    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();
        
        // Create two timers
        {
            let timer1 = registry.timer("timer1");
            sleep(Duration::from_millis(1));
            timer1.stop();
            
            let timer2 = registry.timer("timer2");
            sleep(Duration::from_millis(2));
            timer2.stop();
        }
        
        // Check summary
        let summary = registry.summary();
        assert_eq!(summary.len(), 2);
        assert!(summary.contains_key("timer1"));
        assert!(summary.contains_key("timer2"));
        
        // Reset
        registry.reset();
        
        // Check that histograms were reset
        let histogram1 = registry.histogram("timer1");
        let hist1 = histogram1.lock().unwrap();
        assert_eq!(hist1.count(), 0);
    }
    
    #[test]
    fn test_histogram_percentiles() {
        let mut hist = Histogram::new();
        
        // Add values from 1 to 100
        for i in 1..=100 {
            hist.observe(i);
        }
        
        // Check percentiles
        // Note: Because of the bucketing, the percentiles won't be exact
        assert!(hist.percentile(50.0).unwrap() >= 32);  // Approx median
        assert!(hist.percentile(95.0).unwrap() >= 64);  // 95th percentile
        assert!(hist.percentile(99.0).unwrap() >= 64);  // 99th percentile
    }
    
    #[test]
    fn test_histogram_merge() {
        let mut hist1 = Histogram::new();
        hist1.observe(10);
        hist1.observe(20);
        
        let mut hist2 = Histogram::new();
        hist2.observe(30);
        hist2.observe(40);
        
        hist1.merge(&hist2);
        
        assert_eq!(hist1.count(), 4);
        assert_eq!(hist1.sum(), 100);
        assert_eq!(hist1.min(), Some(10));
        assert_eq!(hist1.max(), Some(40));
        assert_eq!(hist1.average(), Some(25.0));
    }
    
    #[test]
    fn test_bucket_calculation() {
        let hist = Histogram::new();
        
        assert_eq!(hist.bucket_for(0), 0);
        assert_eq!(hist.bucket_for(1), 1);
        assert_eq!(hist.bucket_for(2), 2);
        assert_eq!(hist.bucket_for(3), 2);
        assert_eq!(hist.bucket_for(4), 4);
        assert_eq!(hist.bucket_for(7), 4);
        assert_eq!(hist.bucket_for(8), 8);
        assert_eq!(hist.bucket_for(100), 64);
        assert_eq!(hist.bucket_for(1000), 1024);
    }
}

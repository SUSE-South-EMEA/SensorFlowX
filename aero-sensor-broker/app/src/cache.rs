// cache.rs

// This module defines a `Cache` struct for managing a collection of `DataPoint` instances
// in a thread-safe manner. The cache supports adding new data points, periodically flushing
// the cached data to an InfluxDB instance, and maintaining a maximum cache size.
// It uses an asynchronous approach to handle operations in a non-blocking way, suitable for
// concurrent environments.

use crate::influxdb::InfluxDBManager;
use influxdb2::models::DataPoint;
use log::{debug, error};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct Cache {
    inner: Arc<Mutex<VecDeque<DataPoint>>>,
    max_size: usize,
}

impl Cache {
    // Creates a new Cache instance with a specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }

    // Adds a collection of data points to the cache
    pub async fn add(&self, data_points: Vec<DataPoint>) {
        debug!("Adding {:?} data points to cache", data_points);
        let mut cache = self.inner.lock().await;

        // Remove oldest entries if necessary to make room for new data points
        while cache.len() + data_points.len() > self.max_size {
            cache.pop_front();
        }

        // Add new data points to the end of the cache
        cache.extend(data_points.clone());
    }

    // Retrieves all cached data points and clears the cache
    pub async fn retrieve_and_clear(&self) -> Vec<DataPoint> {
        self.inner.lock().await.drain(..).collect()
    }

    // Periodically flushes the cache to InfluxDB
    pub async fn periodic_flush(
        &self,
        influxdb_manager: InfluxDBManager,
        bucket: &str,
        interval: Duration,
    ) {
        loop {
            sleep(interval).await;

            // Retrieve and clear the cache
            let points_to_flush = self.retrieve_and_clear().await;

            // Skip processing if the cache is empty
            if points_to_flush.is_empty() {
                continue;
            }

            // Write data to InfluxDB and handle potential errors
            if let Err(e) = influxdb_manager.write_data(bucket, points_to_flush).await {
                error!("Failed to flush cache to InfluxDB: {}", e);
            }
        }
    }
}

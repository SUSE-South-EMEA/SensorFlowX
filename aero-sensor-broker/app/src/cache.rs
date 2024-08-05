// cache.rs

use crate::influxdb::InfluxDBManager;

use influxdb2::models::DataPoint;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use log::{debug, error};

#[derive(Clone)]
pub struct Cache {
    inner: Arc<Mutex<VecDeque<DataPoint>>>,
    max_size: usize,
}

impl Cache {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }

    // Add data to the cache
    pub async fn add(&self, data_points: Vec<DataPoint>) {
        debug!("Adding {:?} data points to cache", data_points);
        let mut cache = self.inner.lock().await;

        // Remove oldest entries if necessary to make room for new data points
        while cache.len() + data_points.len() > self.max_size {
            cache.pop_front();
        }

        // Add new data points to the cache
        cache.extend(data_points.clone());
    }

    // Retrieve all cached data and clear the cache
    pub async fn retrieve_and_clear(&self) -> Vec<DataPoint> {
        self.inner.lock().await.drain(..).collect() // Drains all elements and returns them as Vec
    }

    // Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.inner.lock().await.is_empty()
    }

    // Periodically flush the cache to InfluxDB
    pub async fn periodic_flush(
        &self,
        influxdb_manager: InfluxDBManager,
        bucket: &str,
        interval: Duration,
    ) {
        loop {
            sleep(interval).await;

            if !self.is_empty().await {
                let points_to_flush = self.retrieve_and_clear().await;
                if let Err(e) = influxdb_manager
                    .write_data(bucket, points_to_flush)
                    .await
                {
                    error!("Failed to flush cache to InfluxDB: {}", e);
                }
            }
        }
    }
}

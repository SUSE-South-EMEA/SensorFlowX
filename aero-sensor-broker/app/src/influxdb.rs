// influxdb.rs
//
// Handles interactions with InfluxDB. This module encapsulates all the logic needed to initialize
// connections, perform health checks, and write data to InfluxDB. It's designed to abstract the
// complexities of database operations from the main application logic.

use crate::config::InfluxDBConfig;
use crate::errors::AppError;

use chrono::Utc;
use influxdb2::{
    models::{health::Status, DataPoint},
    Client,
};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use log::{debug, error, info};

#[derive(Clone)]
pub struct InfluxDBManager {
    pub client: Arc<Mutex<Client>>,
}

impl InfluxDBManager {
    // Establishes a new client for communicating with InfluxDB using provided configuration settings.
    pub fn new(config: &InfluxDBConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = Client::new(&config.url, &config.org, &config.auth_token);
        info!("New InfluxDB client created for URL: {}", &config.url);
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    // Checks the health of the InfluxDB connection and handles any connectivity issues.
    pub async fn check_health(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = self.client.lock().await;
        match client.health().await {
            Ok(health) if health.status == Status::Pass => {
                info!("InfluxDB health check successful");
                Ok(())
            }
            Ok(health) => {
                error!("InfluxDB health check failed: {:?}", health);
                Err(Box::new(AppError::new("InfluxDB health check failed"))
                    as Box<dyn Error + Send + Sync>)
            }
            Err(e) => {
                error!("Error performing InfluxDB health check: {}", e);
                Err(Box::new(e) as Box<dyn Error + Send + Sync>)
            }
        }
    }

    // Writes sensor data to InfluxDB. It ensures that data points are correctly formatted and sent to the database.
    pub async fn write_data(
        &self,
        bucket: &str,
        points: Vec<DataPoint>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = self.client.lock().await;

        // Attempt to write data points to InfluxDB
        match client.write(bucket, futures::stream::iter(points)).await {
            Ok(_) => {
                debug!("Data written to InfluxDB successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to write data to InfluxDB: {}", e);
                Err(Box::new(e) as Box<dyn Error + Send + Sync>)
            }
        }
    }
}

// Parses sensor data from a formatted string and creates a set of data points for InfluxDB.
pub fn parse_sensor_data(input: String) -> Result<Vec<DataPoint>, Box<dyn Error + Send + Sync>> {
    // Sanitize and split the input data.
    let parts: Vec<f64> = input
        .trim()
        .trim_matches(|c: char| c == '<' || c == '>')
        .split('|')
        .map(str::trim)
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    // Use pattern matching to validate and destructure the parts directly.
    match parts.as_slice() {
        [temperature, humidity, air_quality] => {
            debug!(
                "Data {}, {}, {} parsed successfully from input: {}",
                temperature, humidity, air_quality, input
            );
            let timestamp = Utc::now().timestamp_nanos_opt().unwrap();
            let points = vec![
                DataPoint::builder("temperature")
                    .field("value", *temperature)
                    .timestamp(timestamp)
                    .build()?,
                DataPoint::builder("humidity")
                    .field("value", *humidity)
                    .timestamp(timestamp)
                    .build()?,
                DataPoint::builder("air_quality")
                    .field("value", *air_quality)
                    .timestamp(timestamp)
                    .build()?,
            ];
            Ok(points)
        }
        _ => {
            error!(
                "Incorrect data format or incomplete data in input: {}",
                input
            );
            Err(Box::new(AppError::new(
                "Incorrect data format or incomplete data",
            )))
        }
    }
}

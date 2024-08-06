// influxdb.rs
//
// Handles interactions with InfluxDB. This module encapsulates all the logic needed to initialize
// connections, perform health checks, and write data to InfluxDB. It's designed to abstract the
// complexities of database operations from the main application logic.

use crate::config::InfluxDBConfig;

use influxdb2::{
    models::{health::Status, DataPoint},
    Client,
};
use std::error::Error;

use log::{debug, error, info};

#[derive(Clone)]
pub struct InfluxDBManager {
    pub client: Client,
}

impl InfluxDBManager {
    // Establishes a new client for communicating with InfluxDB using provided configuration settings.
    pub fn new(config: &InfluxDBConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = Client::new(&config.url, &config.org, &config.auth_token);
        info!("New InfluxDB client created for URL: {}", &config.url);
        Ok(Self { client })
    }

    // Checks the health of the InfluxDB connection and handles any connectivity issues.
    pub async fn check_health(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self.client.health().await {
            Ok(health) if health.status == Status::Pass => {
                info!("InfluxDB health check successful");
                Ok(())
            }
            Ok(health) => {
                error!("InfluxDB health check failed: {:?}", health);
                Err("InfluxDB health check failed".into())
            }
            Err(e) => {
                error!("Error performing InfluxDB health check: {}", e);
                Err(e.into())
            }
        }
    }

    // Writes sensor data to InfluxDB. It ensures that data points are correctly formatted and sent to the database.
    pub async fn write_data(
        &self,
        bucket: &str,
        points: Vec<DataPoint>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Attempt to write data points to InfluxDB
        match self
            .client
            .write(bucket, futures::stream::iter(points))
            .await
        {
            Ok(_) => {
                debug!("Data written to InfluxDB successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to write data to InfluxDB: {}", e);
                Err(e.into())
            }
        }
    }
}

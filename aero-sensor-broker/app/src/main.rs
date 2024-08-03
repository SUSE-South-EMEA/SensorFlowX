// main.rs
//
// This is the entry point of the Aero Sensor Flow application. It initializes the logging,
// loads settings from configuration, and manages the lifecycle of the application components
// including the ArduinoManager for handling Arduino device interactions and the InfluxDBManager
// for database operations. The application also establishes an HTTP server for health checks.

mod arduino;
mod config;
mod errors;
mod influxdb;
mod routes;

use arduino::ArduinoManager;
use config::load_settings;
use influxdb::parse_sensor_data;
use influxdb::InfluxDBManager;
use routes::create_health_route;

use std::error::Error;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use log::{debug, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Load settings from the configuration file
    let settings = match load_settings() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load settings: {}", e);
            return;
        }
    };

    // Setup ArduinoManager with settings from the config
    let arduino_manager = match ArduinoManager::new(&settings.arduino) {
        Ok(manager) => Arc::new(Mutex::new(manager)),
        Err(e) => {
            error!("Failed to initialize ArduinoManager: {}", e);
            return;
        }
    };

    // Setup InfluxDBManager with settings from the config
    let influxdb_manager = match InfluxDBManager::new(&settings.influxdb) {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize InfluxDBManager: {}", e);
            return;
        }
    };

    // Initialize the HTTP server for health checks
    let health_route = create_health_route(arduino_manager.clone(), influxdb_manager.clone());
    tokio::spawn(async move {
        warp::serve(health_route).run(([0, 0, 0, 0], 3030)).await;
    });

    // Process data from Arduino and write to InfluxDB in a loop
    if let Err(e) =
        run_serial_to_influx_loop(arduino_manager, influxdb_manager, &settings.influxdb.bucket)
            .await
    {
        error!("Error in serial to InfluxDB loop: {}", e);
    }
}

async fn run_serial_to_influx_loop(
    arduino_manager: Arc<Mutex<ArduinoManager>>,
    influxdb_manager: InfluxDBManager,
    bucket: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Retrieve the environment variable `CLUSTER_DISPLAY_NAME` and use it as a location
    let location = match env::var("CLUSTER_DISPLAY_NAME") {
        Ok(value) => value,
        Err(e) => {
            println!("Couldn't read CLUSTER_DISPLAY_NAME: {}", e);
            String::from("Default")
        }
    };

    loop {
        let data = arduino_manager
            .lock()
            .await
            .read_data()
            .await
            .map_err(|e| {
                error!("Failed to read data from Arduino: {}", e);
                e
            })?;

        let points = parse_sensor_data(data, &location).map_err(|e| {
            error!("Failed to parse sensor data: {}", e);
            e
        })?;

        influxdb_manager
            .write_data(bucket, points)
            .await
            .map_err(|e| {
                error!("Failed to write data to InfluxDB: {}", e);
                e
            })?;

        debug!("Data processed successfully.");
        sleep(Duration::from_millis(1000)).await;
    }
}

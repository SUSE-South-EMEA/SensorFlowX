// main.rs
//
// This is the entry point of the Aero Sensor Flow application. It initializes the logging,
// loads settings from configuration, and manages the lifecycle of the application components
// including the ArduinoManager for handling Arduino device interactions and the InfluxDBManager
// for database operations. The application also establishes an HTTP server for health checks.

mod arduino;
mod cache;
mod config;
mod errors;
mod influxdb;
mod routes;

use cache::Cache;
use arduino::ArduinoManager;
use influxdb::InfluxDBManager;
use influxdb::parse_sensor_data;
use routes::create_health_route;
use config::load_settings;

use std::env;
use std::error::Error;
use tokio::time::{sleep, Duration};

use log::{debug, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    // Load settings from the configuration file
    let settings = load_settings().unwrap_or_else(|e| {
        error!("Failed to load settings: {}", e);
        std::process::exit(1);
    });

    // Setup ArduinoManager with settings from the config
    let arduino_manager = 
        ArduinoManager::new(&settings.arduino).unwrap_or_else(|e| {
            error!("Failed to initialize ArduinoManager: {}", e);
            std::process::exit(1);
    });

    // Initialize Cache
    let cache = Cache::new(1000);

    // Setup InfluxDBManager with settings from the config
    let influxdb_manager = 
        InfluxDBManager::new(&settings.influxdb).unwrap_or_else(|e| {
            error!("Failed to initialize InfluxDBManager: {}", e);
            std::process::exit(1);
    });

    // Initialize the HTTP server for health checks
    let health_route = create_health_route(arduino_manager.clone(), influxdb_manager.clone());
    tokio::spawn(async move {
        warp::serve(health_route).run(([0, 0, 0, 0], 3030)).await;
    });


    // Spawn a task for periodic cache flush to InfluxDB
    tokio::spawn( {
        let cache_to_flush = cache.clone();
        let influxdb_manager_to_flush = influxdb_manager.clone();
        async move {
        cache_to_flush
            .periodic_flush(
                influxdb_manager_to_flush,
                &settings.influxdb.bucket,
                Duration::from_secs(60),
            )
            .await;
        }
    });

    // Process data from Arduino and write to Cache in a loop
    if let Err(e) = run_serial_to_influx_loop(arduino_manager, cache).await {
        error!("Error in serial to InfluxDB loop: {}", e);
    }
}

async fn run_serial_to_influx_loop(
    arduino_manager: ArduinoManager,
    cache: Cache,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Retrieve the environment variable `CLUSTER_DISPLAY_NAME` and use it as a location
    let location = env::var("CLUSTER_DISPLAY_NAME").unwrap_or_else(|e| {
        println!("Couldn't read CLUSTER_DISPLAY_NAME: {}", e);
        String::from("Default")
    });

    loop {
        let data = arduino_manager
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

        cache.add(points).await;

        debug!("Data processed successfully.");
        sleep(Duration::from_millis(1000)).await;
    }
}

// routes.rs
//
// This module defines the HTTP routes for the application, particularly for health checks
// that verify the status of the Arduino connection and the InfluxDB connection.

use crate::arduino::ArduinoManager;
use crate::influxdb::InfluxDBManager;

use serde_json::json;
use warp::{reply, Filter};

// Creates an HTTP route for health checks.
pub fn create_health_route(
    arduino_manager: ArduinoManager,
    influxdb_manager: InfluxDBManager,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and(with_arduino_manager(arduino_manager))
        .and(with_influxdb_manager(influxdb_manager))
        .and_then(handle_health)
}

fn with_arduino_manager(
    arduino_manager: ArduinoManager,
) -> impl Filter<Extract = (ArduinoManager,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || arduino_manager.clone())
}

fn with_influxdb_manager(
    influxdb_manager: InfluxDBManager,
) -> impl Filter<Extract = (InfluxDBManager,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || influxdb_manager.clone())
}

async fn handle_health(
    arduino_manager: ArduinoManager,
    influxdb_manager: InfluxDBManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    let arduino_health = arduino_manager.check_health().await;
    let influxdb_health = influxdb_manager.check_health().await;

    let status = match (arduino_health, influxdb_health) {
        (Ok(_), Ok(_)) => "healthy",
        _ => "unhealthy",
    };

    Ok(reply::json(&json!({"status": status})))
}

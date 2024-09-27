// arduino.rs
//
// Manages the interaction with Arduino devices. It handles serial communication to read sensor data
// from an Arduino and validates the data's format. This module is critical for ensuring data integrity
// before it is forwarded to the database.

use crate::config::ArduinoConfig;

use chrono::Utc;
use serialport::{available_ports, SerialPort, SerialPortType};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use serde_json::Value;

use log::{debug, error, info, warn};

#[derive(Clone)]
pub struct ArduinoManager {
    pub port: Arc<Mutex<Box<dyn SerialPort + Send>>>,
}

impl ArduinoManager {
    // Attempts to connect to an Arduino device based on configuration settings. It will validate
    // the connection by matching the configured product name with available serial ports.
    pub fn new(config: &ArduinoConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let port = find_and_validate_arduino(&config)?;
        info!(
            "New Arduino serial client created for port: {}",
            port.name().unwrap()
        );

        let manager = Self {
            port: Arc::new(Mutex::new(port)),
        };

        let timestamp_ms = Utc::now().timestamp_millis() as i64;

        // Set the time on the Arduino
        match manager.set_time(timestamp_ms) {
            Ok(_) => info!("Successfully set time on Arduino."),
            Err(e) => {
                error!("Failed to set time on Arduino: {}", e);
                return Err(e);
            }
        }

        Ok(manager)
    }

    // Function to set time on Arduino synchronously
    fn set_time(&self, timestamp: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut port = self.port.try_lock().expect("Failed to lock port");
        // Send the SET_TIME command followed by a newline
        port.write_all(b"SET_TIME\n")?;
        port.flush()?; // Ensure the command is sent immediately

        // Convert the timestamp to a string and send it followed by a newline
        port.write_all(format!("{}\n", timestamp).as_bytes())?;
        port.flush()?;

        // We wait for a simple acknowledgment from the Arduino.
        let mut buffer = vec![0; 1024];
        port.read_exact(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer).trim().to_string();
        debug!("{}", response);

        if response.contains(&format!("New timestamp received and set: {}", timestamp)) {
            debug!("Timestamp set successfully");
            Ok(())
        } else {
            error!("Failed to set timestamp, received: '{}'", response);
            Err("Failed to set timestamp".into())
        }
    }

    // Reads data from the Arduino. This function continuously checks for new data,
    // validates its format, and returns the data if it's correctly formatted.
    pub async fn read_data(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        loop {
            match self.try_read_data().await {
                Ok(Some(data_string)) if self.is_valid_data(&data_string) => {
                    debug!("Received valid data: '{}'", data_string);
                    return Ok(data_string);
                }
                Ok(Some(data_string)) => {
                    warn!("Invalid data format: '{}'", data_string);
                }
                Ok(None) => {
                    debug!("No data available; will check again after delay.");
                    sleep(Duration::from_millis(1000)).await;
                }
                Err(e) => {
                    error!("Error reading data: {}", e);
                    return Err(e);
                }
            }
        }
    }

    async fn try_read_data(&self) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let mut port = self.port.lock().await;
        match port.bytes_to_read() {
            Ok(available_bytes) if available_bytes > 0 => {
                let mut buffer = vec![0; available_bytes as usize];
                port.read_exact(&mut buffer)?;
                let data_string = String::from_utf8(buffer)?.trim().to_string();
                Ok(Some(data_string))
            }
            Ok(_) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn is_valid_data(&self, data: &str) -> bool {
        // Try to parse the input string as a JSON array
        let parsed_data: Result<Value, _> = serde_json::from_str(data);

        match parsed_data {
            Ok(Value::Array(arr)) => {
                // Iterate over each item in the array and check its structure
                for item in arr {
                    if let Value::Object(obj) = item {
                        // Check for "type", "value" fields
                        if !obj.contains_key("type") || !obj.contains_key("value") {
                            return false;
                        }
                    } else {
                        return false; // Not an object, invalid
                    }
                }
                true // All items are valid
            }
            _ => false, // Not a valid JSON array
        }
    }

    pub async fn check_health(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut port = self.port.lock().await;
        port.write_all(b"PING\n")?;
        port.flush()?;
        sleep(Duration::from_millis(100)).await;

        let mut buffer = String::new();
        let mut reader = BufReader::new(&mut *port);
        reader.read_line(&mut buffer)?;

        match buffer.trim() {
            "PONG" => {
                info!("Arduino health check successful");
                Ok(())
            }
            _ => {
                error!("Arduino Health check failed");
                Err("Arduino Health check failed".into())
            }
        }
    }
}

fn find_and_validate_arduino(
    config: &ArduinoConfig,
) -> Result<Box<dyn SerialPort>, Box<dyn Error + Send + Sync>> {
    let target_product = config.device_name.as_str();
    let ports = available_ports().map_err(|e| Box::<dyn Error + Send + Sync>::from(e))?;

    debug!("Available ports: {:?}", ports);

    let arduino_port = ports
        .iter()
        .find(|p| {
            if let SerialPortType::UsbPort(ref info) = p.port_type {
                if let Some(product) = &info.product {
                    let product_normalized = normalize_product_name(product);
                    debug!(
                        "Checking port: {:?}, normalized product: {}",
                        p, product_normalized
                    );
                    return product_normalized == normalize_product_name(target_product);
                }
            }
            false
        })
        .ok_or_else(|| {
            error!("Arduino not found");
            Box::new(std::fmt::Error) as Box<dyn Error + Send + Sync>
        })?;

    debug!("Arduino found on port: {}", arduino_port.port_name);

    serialport::new(&arduino_port.port_name, config.baud_rate)
        .timeout(Duration::from_millis(config.timeout))
        .open()
        .map_err(|e| e.into())
        .map(|port| {
            debug!("Successfully opened port: {}", arduino_port.port_name);
            port
        })
}

// Normalize product names by removing spaces, underscores, hyphens, and converting to lowercase
fn normalize_product_name(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "")
        .replace("_", "")
        .replace("-", "")
}

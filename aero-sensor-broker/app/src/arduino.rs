// arduino.rs
//
// Manages the interaction with Arduino devices. It handles serial communication to read sensor data
// from an Arduino and validates the data's format. This module is critical for ensuring data integrity
// before it is forwarded to the database.

use crate::config::ArduinoConfig;

use serialport::{available_ports, SerialPort, SerialPortType};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

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
        Ok(Self {
            port: Arc::new(Mutex::new(port)),
        })
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
        data.starts_with('<') && data.ends_with('>')
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
                debug!("Health check successful");
                Ok(())
            }
            _ => {
                error!("Health check failed");
                Err("Health check failed".into())
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

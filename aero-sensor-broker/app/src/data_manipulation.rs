// data_manipulation.rs
//
// This module processes a collection of MyDataPoints, which are custom data points containing measurements,
// tags, fields, and timestamps. The goal is to:
// 1. Group the data points by their measurement type.
// 2. Filter out any data points that do not have both a field value and a timestamp.
// 3. Calculate the average value and timestamp for each group of data points.
// 4. Build new DataPoint instances from these averages, maintaining the original tags.
//
// This process helps in reducing the amount of data sent to InfluxDB by summarizing it.

use chrono::Utc;
use influxdb2::models::{DataPoint, FieldValue};
use log::{debug, error, trace};
use std::collections::BTreeMap;
use std::error::Error;

/// Represents a custom data point.
#[derive(Debug, Clone)]
pub struct MyDataPoint {
    measurement: String,
    tags: BTreeMap<String, String>,
    fields: BTreeMap<String, FieldValue>,
    timestamp: Option<i64>,
}

impl MyDataPoint {
    pub fn new(
        measurement: String,
        tags: BTreeMap<String, String>,
        field: FieldValue,
        timestamp: i64,
    ) -> Self {
        Self {
            measurement,
            tags,
            fields: BTreeMap::from([("value".into(), field)]),
            timestamp: Some(timestamp),
        }
    }

    pub fn get_measurement(&self) -> &str {
        &self.measurement
    }

    pub fn get_field_value(&self) -> Option<f64> {
        if let Some(FieldValue::F64(value)) = self.fields.get("value") {
            Some(*value)
        } else {
            None
        }
    }

    pub fn get_timestamp(&self) -> Option<i64> {
        self.timestamp
    }

    pub fn get_tags(&self) -> BTreeMap<String, String> {
        self.tags.clone()
    }
}

/// Groups and filters data points by measurement type.
fn group_and_filter_data_points(
    data_points: Vec<MyDataPoint>,
) -> BTreeMap<String, Vec<MyDataPoint>> {
    let valid_points: Vec<MyDataPoint> = data_points
        .into_iter()
        .filter(|point| {
            let is_valid = point.get_field_value().is_some()
                && point.get_timestamp().is_some()
                && !point.get_measurement().is_empty();
            trace!("Filtering point: {:?}, valid: {}", point, is_valid);
            is_valid
        })
        .collect();

    valid_points
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, point| {
            acc.entry(point.get_measurement().to_string())
                .or_insert_with(Vec::new)
                .push(point);
            acc
        })
}

/// Calculates the average value and timestamp for a group of data points.
fn calculate_average_for_group(points: &[MyDataPoint]) -> Option<(f64, i64)> {
    let count = points.len() as f64;

    // Handle case with no data points
    if count == 0.0 {
        return None;
    }

    let average_value = points
        .iter()
        .filter_map(|p| p.get_field_value())
        .sum::<f64>()
        / count;

    let average_timestamp = points
        .iter()
        .filter_map(|p| p.get_timestamp())
        .map(|ts| ts as f64)
        .sum::<f64>() as i64
        / count as i64;

    debug!(
        "Calculated averages - Value: {}, Timestamp: {} for {} points",
        average_value, average_timestamp, count
    );

    Some((average_value, average_timestamp))
}

/// Creates a new averaged DataPoint from a group of MyDataPoints.
fn create_averaged_data_point(
    measurement: &str,
    average_value: f64,
    average_timestamp: i64,
    tags: BTreeMap<String, String>,
) -> DataPoint {
    let builder = DataPoint::builder(measurement)
        .field("value", average_value)
        .timestamp(average_timestamp);

    tags.iter()
        .fold(builder, |builder, (key, value)| builder.tag(key, value))
        .build()
        .unwrap()
}

/// Main function to calculate average data points from a vector of MyDataPoints.
pub fn calculate_average(data_points: Vec<MyDataPoint>) -> Vec<DataPoint> {
    let grouped_points = group_and_filter_data_points(data_points);

    grouped_points
        .into_iter()
        .filter(|(_, points)| !points.is_empty())
        .filter_map(|(measurement, points)| {
            debug!("Averaging points for measurement: {}", measurement);
            match calculate_average_for_group(&points) {
                Some((average_value, average_timestamp)) => {
                    debug!(
                        "Calculated average - Measurement: {}, Average Value: {}, Average Timestamp: {}",
                        measurement, average_value, average_timestamp
                    );

                    let first_point = points.first()?;
                    Some(create_averaged_data_point(
                        &measurement,
                        average_value,
                        average_timestamp,
                        first_point.get_tags(),
                    ))
                }
                None => {
                    debug!("No valid points for measurement: {}", measurement);
                    None
                }
            }
        })
        .collect()
}

/// Parses sensor data from a formatted string and creates a set of data points for InfluxDB.
pub fn parse_sensor_data(
    input: String,
    location: &str,
) -> Result<Vec<MyDataPoint>, Box<dyn Error + Send + Sync>> {
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

            let tags = BTreeMap::from([("location".into(), location.into())]);

            let points: Vec<MyDataPoint> = vec![
                MyDataPoint::new(
                    "temperature".into(),
                    tags.clone(),
                    FieldValue::from(*temperature),
                    timestamp,
                ),
                MyDataPoint::new(
                    "humidity".into(),
                    tags.clone(),
                    FieldValue::from(*humidity),
                    timestamp,
                ),
                MyDataPoint::new(
                    "air_quality".into(),
                    tags,
                    FieldValue::from(*air_quality),
                    timestamp,
                ),
            ];
            Ok(points)
        }
        _ => {
            error!(
                "Incorrect data format or incomplete data in input: {}",
                input
            );
            Err("Incorrect data format or incomplete data".into())
        }
    }
}

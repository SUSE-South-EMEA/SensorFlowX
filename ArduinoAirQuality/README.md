# Arduino Air Quality Sensor Application README

## Introduction

This Arduino application is designed to gather air quality data from environmental sensors, specifically temperature, humidity, and CO2 levels. It utilizes a DHT22 sensor for temperature and humidity readings and an MQ135 sensor for CO2 concentration measurement. The application sends these readings through the serial port at regular intervals, formatted for easy parsing and integration with external systems.

## Key Features

- **Sensor Integration**: Utilizes DHT22 and MQ135 sensors to measure environmental conditions.
- **Serial Communication**: Outputs sensor data in a structured format over the serial connection.
- **Health Check**: Responds to health check commands to verify device connectivity and functionality.

## Hardware Requirements

- **Arduino Board**: Compatible with standard Arduino boards such as Arduino Uno.
- **DHT22 Sensor**: Measures ambient temperature and humidity.
- **MQ135 Sensor**: Measures air quality concerning CO2 concentration.
- **Connecting Wires**: Appropriate wires to connect sensors to the Arduino board:
  -  MQ135 Air Quality Sensor:
      Connect the MQ135 sensor's analog output pin to the A0 pin on the Arduino. This pin is used to read the analog voltage corresponding to the detected gas concentration.
  - DHT22 Temperature and Humidity Sensor:
    Connect the DHT22 sensor's data pin to Digital Pin 2 on the Arduino. This digital pin is configured to handle the sensor's data output.

## Software Setup

### Prerequisites

Ensure you have the Arduino IDE installed on your computer to upload the sketch to the Arduino board. This program requires the following libraries:
- `DHT sensor library`: Manages the DHT22 sensor data reading.
- `MQ135 library`: Manages the air quality sensor data reading.

You can install these libraries through the Arduino IDE Library Manager.

### Installation

1. **Connect the Arduino to your computer** via a USB cable.
2. **Open the Arduino IDE** and navigate to the sketch file.
3. **Install the necessary libraries** if not already installed:
   - Go to Sketch > Include Library > Manage Libraries.
   - Search for "DHT" and "MQ135" and install the libraries.
4. **Select the correct board and port**:
   - Tools > Board: "Arduino AVR Boards" > (select your Arduino model)
   - Tools > Port: (select the port that lists your Arduino)
5. **Upload the sketch** to the Arduino by clicking the upload button in the IDE.

## Usage

Once the application is uploaded and running, it will send sensor readings formatted as `<temperature|humidity|CO2>` every second. If the Arduino receives a "PING" command over the serial interface, it responds with "PONG" and skips the regular data transmission in that cycle.

This functionality allows for integration with systems capable of reading serial data, such as a computer running a broker application that parses and forwards this data to a database or monitoring system.

## Monitoring and Troubleshooting

- **Serial Monitor**: Open the serial monitor in the Arduino IDE to view the output and ensure the data is being sent correctly.
- **Health Check**: Send a "PING" from your serial terminal, and expect a "PONG" response to confirm the system's responsiveness.

# Aero Sensor Broker README

## Introduction

The Aero Sensor Broker is an application engineered to facilitate the collection of air quality data from Arduino-equipped sensors and store this information in InfluxDB. This solution is particularly suitable for applications requiring real-time environmental monitoring.

## Key Features

- **Data Acquisition**: Captures real-time measurements such as temperature, humidity, and CO2 levels from Arduino sensors.
- **Persistent Storage**: Automatically stores the sensor readings in InfluxDB for trend analysis and historical review.
- **Health Monitoring**: Implements a health-check mechanism to ensure ongoing operational status of the Arduino connection and the InfluxDB service.

## System Requirements

- **Development Tools**: Rust and Cargo for compiling the application.
- **Hardware**: Arduino with appropriate sensors for measuring environmental parameters.
- **Database**: Access to an InfluxDB instance for data storage.

## Installation and Execution

### Local Setup

1. **Compile the Application**:
   Use Cargo to build the application from source:
   ```bash
   $ cargo build
   ```
   This command compiles the source code into an executable.

2. **Run the Application**:
   Start the application by running the generated executable:
   ```bash
   $ ./target/debug/aero-sensor-broker
   ```
   This launches the broker, beginning data collection and storage.

### Containerization with Podman

1. **Build the Container Image**:
   Create a Docker-compatible image using Podman:
   ```bash
   $ podman build . -t aero-sensor-broker:0.1
   ```
   This step packages the application and its dependencies into a container for deployment.

## Deployment on Kubernetes

### Configuration

Before deploying, ensure the `Settings.toml` configuration file is correctly set up with your Arduino and InfluxDB settings. This file must be accessible within the container and may be mounted via Kubernetes secrets or config maps.

### Kubernetes Deployment

We will need the application configuration:
```bash
$ kubectl create secret generic influxdb-config --from-file=settings/Settings.toml
```

Deploy the application within a Kubernetes environment using the provided deployment descriptor:

```bash
$ helm install aero-sensor-broker ./kubernetes/helm-chart/
```

This deployment script configures the broker as a Kubernetes Deployment object, ensuring it is deployed with the necessary privileges and resource requests to function correctly. It also sets up readiness and liveness probes to manage the application lifecycle based on its health status.

## Health Monitoring

The application features a `/healthz` HTTP endpoint for health checks. This endpoint is used by Kubernetes to assess the readiness and liveness of the application. It ensures that both the Arduino device connection and the InfluxDB connection are active and functioning correctly.


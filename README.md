# Aero Sensor Monitoring Project

This project offers a complete solution for monitoring air quality using an Arduino-based sensor setup and a Rust-based Aero Sensor Broker application. The system is designed to be deployed on Kubernetes clusters, with capabilities for local data caching, aggregation, and efficient data storage and visualization using InfluxDB and Grafana.

## Project Structure

- **ArduinoAirQuality**: Contains the Arduino code responsible for collecting air quality data such as temperature, humidity, and CO2 levels.
- **AeroSensorBroker**: Contains the Rust application code that reads the sensor data from the Arduino device, processes it, and sends it to InfluxDB.

## Prerequisites

- **Arduino UNO WiFi R4 CMSIS-DAP** with DHT22 and MQ135 sensors.
- **Kubernetes cluster** managed by Rancher.
- **Akri** installed on your Kubernetes cluster to handle device discovery.
- **InfluxDB** instance for data storage.
- **Grafana** for visualizing the collected data.

## Step 1: Setting Up the Arduino Air Quality Sensor

1. **Upload the Arduino Code**:
   - The code is located in the `ArduinoAirQuality` folder.
   - This code collects temperature, humidity, and CO2 data and sends it over serial communication.

2. **Connect the Arduino**:
   - Ensure the Arduino is connected to your Kubernetes node where Akri will discover it.

## Step 2: Deploying Aero Sensor Broker on Kubernetes

1. **Install Akri**:
   - Akri is required to detect and manage Arduino devices within the Kubernetes cluster.

   ```bash
   helm install akri akri-helm-charts/akri \
     --set kubernetesDistro=k3s \
     --set udev.discovery.enabled=true \
     --set udev.configuration.enabled=true \
     --set udev.configuration.discoveryDetails.udevRules[0]='SUBSYSTEM=="tty", ATTRS{manufacturer}=="Arduino", ATTRS{product}=="UNO WiFi R4 CMSIS-DAP"'
   ```

2. **Deploy InfluxDB**:
   - If you do not already have an InfluxDB instance, you can deploy it using Rancher's application registry.

   ```bash
   kubectl create secret docker-registry application-collection \
     --docker-server=dp.apps.rancher.io \
     --docker-username=<email> \
     --docker-password=<password>

   helm upgrade --install influxdb oci://dp.apps.rancher.io/charts/influxdb \
     --version 2.1.2 \
     --set 'global.imagePullSecrets[0].name=application-collection' \
     --set 'adminUser.password=password' \
     --set 'adminUser.token=token' \
     --set 'adminUser.bucket=sensor_data'
   ```

3. **Deploy Aero Sensor Broker**:
   - The Aero Sensor Broker reads data from the Arduino, processes it, and writes it to InfluxDB.
   - Pre-built images are available for different versions:

     - **Version 0.1**: Basic application.
     - **Version 0.2**: Includes local caching.
     - **Version 0.3**: Includes data aggregation to reduce connectivity requirements.

   You can deploy the desired version using the appropriate tag:

   ```bash
   kubectl apply -f deployment.yaml
   ```

   The `deployment.yaml` should reference the correct image, for example:

   ```yaml
   image: ghcr.io/suse-south-emea/aero-sensor-broker:0.1
   ```

## Step 3: Visualizing Data with Grafana

1. **Apply the Grafana Dashboard**:
   - The dashboard YAML is available and can be applied directly to Kubernetes.

   ```bash
   kubectl apply -f grafana-dashboard-aero-sensor.yaml
   ```

2. **Deploy Grafana** (if needed):
   - If Grafana is not already deployed, you can use Rancher's application registry.

   ```bash
   kubectl create secret docker-registry application-collection \
     --docker-server=dp.apps.rancher.io \
     --docker-username=<email> \
     --docker-password=<password>

   helm upgrade --install my-grafana oci://dp.apps.rancher.io/charts/grafana \
     --version 8.4.0 \
     --set 'global.imagePullSecrets[0].name=application-collection' \
     --set adminPassword='PasswordSuperSicura' \
     --set datasources."datasources\.yaml".apiVersion=1 \
     --set datasources."datasources\.yaml".datasources[0].name=InfluxDB \
     --set datasources."datasources\.yaml".datasources[0].type=influxdb \
     --set datasources."datasources\.yaml".datasources[0].url=http://influxdb:80 \
     --set datasources."datasources\.yaml".datasources[0].access=proxy \
     --set datasources."datasources\.yaml".datasources[0].isDefault=true \
     --set datasources."datasources\.yaml".datasources[0].editable=true \
     --set datasources."datasources\.yaml".datasources[0].jsonData.httpMode=POST \
     --set datasources."datasources\.yaml".datasources[0].jsonData.organization=influxdata \
     --set datasources."datasources\.yaml".datasources[0].jsonData.version=Flux \
     --set datasources."datasources\.yaml".datasources[0].jsonData.defaultBucket=sensor_data \
     --set datasources."datasources\.yaml".datasources[0].secureJsonData.token="token" \
     --set sidecar.dashboards.enabled=true \
     --set sidecar.dashboards.label=grafana_dashboard
   ```

## Versioning and Deployment

- **Version 0.1**: Initial deployment with basic sensor data handling.
- **Version 0.2**: Introduced local data caching to improve reliability in case of network disruptions.
- **Version 0.3**: Added data aggregation to minimize connectivity requirements, demonstrating the power of edge computing.

You can switch between these versions by checking out the appropriate git tag and deploying the associated Docker image:

```bash
git checkout tags/0.2 -b deployment-0.2
```

## Conclusion

This project provides a scalable and flexible platform for air quality monitoring using edge computing techniques. By leveraging Kubernetes, Akri, InfluxDB, and Grafana, it ensures that data is captured, stored, and visualized efficiently, even in distributed environments.

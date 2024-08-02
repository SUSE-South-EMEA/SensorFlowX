#include <DHT.h>
#include <MQ135.h>

const int airQualityPin = A0; // Analog input pin that MQ-135 is attached to
MQ135 airSensor(airQualityPin);

#define DHTPIN 2          // Digital pin connected to the DHT sensor
#define DHTTYPE DHT22     // DHT 22 (AM2302)
DHT dht(DHTPIN, DHTTYPE);

unsigned long previousMillis = 0;  // Stores the last time data was sent
const long interval = 1000;        // Interval at which to send data (milliseconds)

void setup() {
  Serial.begin(9600);
  dht.begin();
    while (!Serial) {
    ; // Wait for serial port to connect. Needed for native USB
  }
}

void loop() {
  unsigned long currentMillis = millis();

  if (Serial.available() > 0) {
    String command = Serial.readStringUntil('\n');
    command.trim();
    
    // Respond to the health check command
    if (command == "PING") {
      Serial.println("PONG");
      return; // Exit to avoid sending regular data
    }
  }
  // Regular data transmission
  if (currentMillis - previousMillis >= interval) {
    previousMillis = currentMillis;

    float temperature = dht.readTemperature();
    float humidity = dht.readHumidity();
    float co2_ppm = airSensor.getCorrectedPPM(temperature, humidity);

    char returnText[100];
    sprintf(returnText, "<%.2f|%.2f|%.2f>", temperature, humidity, co2_ppm);
    Serial.println(returnText);
  }
}

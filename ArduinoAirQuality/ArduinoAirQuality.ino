#include <DHT.h>
#include <MQ135.h>
#include <ArduinoJson.h>

const int airQualityPin = A0;
MQ135 airSensor(airQualityPin, 850, 20);

#define DHTPIN 2
#define DHTTYPE DHT22
DHT dht(DHTPIN, DHTTYPE);

unsigned long previousMillis = 0;
const long interval = 1000; // Interval at which to send data (milliseconds)
unsigned long long initialTimestamp = 0; // To store the initial timestamp in nanoseconds
unsigned long long startTime = 0; // To record the time when the initial timestamp is set

void setup() {
  Serial.begin(115200); // Adjusted for faster communication
  dht.begin();
  while (!Serial) {
    ; // Wait for serial port to connect. Needed for native USB
  }

  Serial.println("Send initial UTC timestamp in nanoseconds or use SET_TIME to update:");
}

void loop() {
  if (Serial.available() > 0) {
    String command = Serial.readStringUntil('\n');
    command.trim();

    if (command == "PING") {
      Serial.println("PONG");
    }
    else if (command.startsWith("SET_TIME")) {
      Serial.println("Send new UTC timestamp in milliseconds:");
      while (!Serial.available()) {
        ; // Wait for the new timestamp
      }
      String timestampStr = Serial.readStringUntil('\n');
      timestampStr.trim();
      initialTimestamp = atoll(timestampStr.c_str());
      startTime = millis(); // Capture the current time when the timestamp is set
      Serial.print("New timestamp received and set: ");
      Serial.println(initialTimestamp);
    }
    return; // Exit the current loop iteration to wait for more commands or continue processing
  }

  unsigned long currentMillis = millis();

  // Check if the current time interval has elapsed
  if (currentMillis - previousMillis >= interval) {
    previousMillis = currentMillis;

    // Calculate the current adjusted timestamp in milliseconds
    unsigned long long elapsed = currentMillis - startTime;
    unsigned long long currentTimestamp = initialTimestamp + elapsed;

    float temperature = dht.readTemperature();
    float humidity = dht.readHumidity();
    float airQuality = airSensor.getCorrectedPPM(temperature, humidity);

    StaticJsonDocument<300> doc;
    JsonArray data = doc.to<JsonArray>();

    JsonObject tempObj = data.createNestedObject();
    tempObj["type"] = "temperature";
    tempObj["value"] = temperature;
    tempObj["timestamp"] = currentTimestamp;

    JsonObject humObj = data.createNestedObject();
    humObj["type"] = "humidity";
    humObj["value"] = humidity;
    humObj["timestamp"] = currentTimestamp;

    JsonObject airObj = data.createNestedObject();
    airObj["type"] = "airquality";
    airObj["value"] = airQuality;
    airObj["timestamp"] = currentTimestamp;

    String output;
    serializeJson(data, output);
    Serial.println(output);
  }
}

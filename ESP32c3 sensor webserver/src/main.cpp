#include <Arduino.h>
#include <WiFi.h>
#include <AsyncTCP.h>
#include <ESPAsyncWebServer.h>
#include <ArduinoJson.h>
#include <secrets.h>

#include <Adafruit_Sensor.h>
#include <DHT.h>
#include <DHT_U.h>

#define DHTPIN 2
#define DHTTYPE DHT22

DHT_Unified dht(DHTPIN, DHTTYPE);
uint32_t delayMS;

AsyncWebServer server(80);

const char* ssid = WIFI_SSID;
const char* password = WIFI_PASSWORD;

struct {
  double Temp;
  double Humid;
} environmentalSample;

void notFound(AsyncWebServerRequest *request) {
    request->send(404, "text/plain", "Not found");
}

void sampleTempHumid() {
    delay(delayMS);
    sensors_event_t event;

    dht.temperature().getEvent(&event);
    environmentalSample.Temp = isnan(event.temperature) ? NAN : event.temperature;

    dht.humidity().getEvent(&event);
    environmentalSample.Humid = isnan(event.relative_humidity) ? NAN : event.relative_humidity;

    Serial.printf("Temperature: %.2f°C\n", environmentalSample.Temp);
    Serial.printf("Humidity: %.2f%%\n", environmentalSample.Humid);
}

void setup() {
    Serial.begin(115200);
    WiFi.mode(WIFI_STA);
    WiFi.begin(ssid, password);
    if (WiFi.waitForConnectResult() != WL_CONNECTED) {
        Serial.println("WiFi Failed!");
        return;
    }
    Serial.print("IP Address: ");
    Serial.println(WiFi.localIP());

    server.on("/", HTTP_GET, [](AsyncWebServerRequest *request){
        request->send(200, "text/plain", "Howdy");
    });

    server.on("/get", HTTP_GET, [](AsyncWebServerRequest *request) {
        sampleTempHumid();
        StaticJsonDocument<128> doc;
        doc["Temp"] = environmentalSample.Temp;
        doc["Humid"] = environmentalSample.Humid;

        String json;
        serializeJson(doc, json);
        request->send(200, "application/json", json);
    });

    server.onNotFound(notFound);
    server.begin();

    /* SETUP DHT22 sensor*/
    dht.begin();
    sensor_t sensor;
    dht.temperature().getSensor(&sensor);
    delayMS = sensor.min_delay / 1000;
}

void checkWiFiConnection() {
    if (WiFi.status() != WL_CONNECTED) {
        Serial.println("WiFi disconnected! Attempting reconnect...");
        WiFi.disconnect();
        WiFi.reconnect();
        unsigned long startAttemptTime = millis();
        // Wait up to 10 seconds for connection
        while (WiFi.status() != WL_CONNECTED && millis() - startAttemptTime < 10000) {
            delay(500);
            Serial.print(".");
        }
        if (WiFi.status() == WL_CONNECTED) {
            Serial.println("\nWiFi reconnected!");
            Serial.print("IP Address: ");
            Serial.println(WiFi.localIP());
        } else {
            Serial.println("\nWiFi reconnection failed.");
        }
    }
}

void loop() {
    checkWiFiConnection();
    delay(60000);
}

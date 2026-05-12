/*
Author: Django Wardlow

Web dashboard component of RHcar library

provides the websight that is used to controll the Rose-Hulman car

it is unlikely that students should modify this file
*/

#ifndef Dashboard_h

#define Dashboard_h

#include <Arduino.h>
#include <WiFi.h>
#include <AsyncTCP.h>
#include <ESPAsyncWebServer.h>
#include "LittleFS.h"
#include <Arduino_JSON.h>
#include <functional>



class Dashboard{

  public:
    Dashboard();
    void init_dashboard(const char* name, std::function<void()> race_start_fn, std::function<void()> race_man_charge_fn, std::function<void()> race_stop_fn);
    void log(String);
    void update_power(float voltage, float current, float watts, float energy, float charged, float available);
    int update_race_data();
    void update_user_vars();

  private:
    //name of car, used for SSID
    const char* name;

    //time the race was started
    unsigned long race_start_time;

    //web server on standerd HTTP port
    AsyncWebServer server;

    //used to send updates to the dashboard
    AsyncEventSource events;

    //used for race time and buttons
    AsyncWebSocket ws;

    //holds log message and converts it into json
    JSONVar JSONdata;

    //used to handle race btn presses in RHcar
    std::function<void()> race_start_fn;
    std::function<void()> race_man_charge_fn;
    std::function<void()> race_stop_fn;

    int clients;
    
    //internal functions
    void initLittleFS();
    void initWiFi(const char* name);
    void initWebSocket();
    void handleWebSocketMessage(void *arg, uint8_t *data, size_t len);
    void onEvent(AsyncWebSocket *server, AsyncWebSocketClient *client, AwsEventType type, void *arg, uint8_t *data, size_t len);


};

#endif
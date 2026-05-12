/*
Author: Django Wardlow

code that implements the web server for the Rose-Hulman car dashboard

uses http for websight, SSE for power and log updates, and websockets for race control / updates

it is unlikely that students should modify this file

//Based on:
// https://randomnerdtutorials.com/esp32-web-server-gauges/
// https://randomnerdtutorials.com/esp32-access-point-ap-web-server/
// https://randomnerdtutorials.com/esp32-websocket-server-arduino/
*/

#include "HardwareSerial.h"
#include <sys/_types.h>
#include "esp32-hal.h"


#include "WString.h"
#include "Dashboard.h"
#include "UserVar.h"

//must init server and events before dashboard
Dashboard::Dashboard() : server(80), events("/events"), ws("/ws"){
}

void Dashboard::init_dashboard(const char* name, std::function<void()> race_start_fn, std::function<void()> race_man_charge_fn, std::function<void()> race_stop_fn){
  this->name = name;

  this->race_start_fn = race_start_fn;
  this->race_man_charge_fn = race_man_charge_fn;
  this->race_stop_fn = race_stop_fn;

  this->race_start_time = 0;

  this->clients = 0;

  this->initWiFi(name);
  this->initLittleFS();
  this->initWebSocket();

  // Web Server Root URL
  server.on("/", HTTP_GET, [](AsyncWebServerRequest *request){
    request->send(LittleFS, "/index.html", "text/html");
  });

  server.serveStatic("/", LittleFS, "/");


  events.onConnect([](AsyncEventSourceClient *client){
    if(client->lastId()){
      Serial.printf("Client reconnected! Last message ID that it got is: %u\n", client->lastId());
    }
    // send event with message "hello!", id current millis
    // and set reconnect delay to 1 second
    client->send("hello!", NULL, millis(), 10000);
  });
  server.addHandler(&events);

  // Start server
  server.begin();
}

// Initialize LittleFS - reads the files that are served from the internal flash
void Dashboard::initLittleFS() {
  if (!LittleFS.begin()) {
    Serial.println("An error has occurred while mounting LittleFS");
  }
  Serial.println("LittleFS mounted successfully");
}

// Initialize WiFi
void Dashboard::initWiFi(const char* ssid) {
  WiFi.mode(WIFI_AP);
  WiFi.softAP(ssid);
  Serial.println("started WiFi AP");
}

//init websocket interface for btns and race time
void Dashboard::initWebSocket() {
  this->ws.onEvent([&](AsyncWebSocket *server, AsyncWebSocketClient *client, AwsEventType type, void *arg, uint8_t *data, size_t len){this->onEvent(server, client, type, arg, data, len);});
  this->server.addHandler(&ws);
}

//sends the message string to the client for display in the log
void Dashboard::log(String msg){
  events.send(msg.c_str(), "log_message", millis());
}

//updates the power readings on the dashboard
//TODO update these slower
void Dashboard::update_power(float voltage, float current, float watts, float energy, float charged, float available){
  this->JSONdata["voltage"] = String(voltage,2);
  this->JSONdata["current"] = String(current,2);
  this->JSONdata["watts"] = String(watts,2);
  this->JSONdata["energy"] = String(energy,2);
  this->JSONdata["charged"] = String(charged,2);
  this->JSONdata["available"] = String(available,2);

  events.send(JSON.stringify(this->JSONdata).c_str(), "pwr_data", millis());
}

void Dashboard::update_user_vars(){
  //update user ints
  String ints = String();

  if(LastUserInt >= 0){
    for(int i = 0; i <= LastUserInt; i++){
      if(i != 0){
            ints += ":";
      }
      ints += String(i);
      ints += ",";
      ints += UserInts[i]->name;
      ints += ",";
      ints += String(UserInts[i]->get());

      // Serial.println(ints);
      // Serial.println("--");

    }
  }
  

  events.send(ints, "user_ints", millis());

  // Serial.println("------------");


  //update user floats

  String floats = String();

  if(LastUserFloat >= 0){
    for(int i = 0; i <= LastUserFloat; i++){
      if(i != 0){
            floats += ":";
      }
      floats += String(i);
      floats += ",";
      floats += UserFloats[i]->name;
      floats += ",";
      floats += String(UserFloats[i]->get(), 5);

      // Serial.println(floats);
      // Serial.println("--");

    }
  }

    // Serial.println("------------");

  events.send(floats, "user_floats", millis());

}

//updates the race time varable and returns it
//TODO dont use websockets for this
//probably just switch away from websockets all together as it does nto work very well
int Dashboard::update_race_data(){

  unsigned long current_time = millis();

  int race_time = (current_time-(this->race_start_time))/1000;

  // Serial.println(race_time);

  int clients = this->clients;

  if (clients > 0){

    //TODO run this less often to save compute
    this->ws.cleanupClients(10);

    this->ws.textAll(String(race_time));

  }

  return race_time;
  
}


void Dashboard::handleWebSocketMessage(void *arg, uint8_t *data, size_t len) {
  AwsFrameInfo *info = (AwsFrameInfo*)arg;
  if (info->final && info->index == 0 && info->len == len && info->opcode == WS_TEXT) {
    data[len] = 0;
    if (strcmp((char*)data, "start") == 0) {
      this->race_start_time = millis();
      (this->race_start_fn)();
    }
    else if (strcmp((char*)data, "charge") == 0) {
      (this->race_man_charge_fn)();
    }
    else if (strcmp((char*)data, "stop") == 0) {
      (this->race_stop_fn)();

    }

    //update user vars

    char* data1 = (char*)data;

    // Serial.println(data1);

    String data2 = String(data1);

    // Serial.println(data2);

    String sub = data2.substring(0, 3);

    // Serial.println(sub);

    //update an int var
    if (sub == "int") {
      // Serial.println("INT");

      //parse index and value
      int comma = data2.lastIndexOf(",");
      String index = data2.substring(4, comma);
      String val = data2.substring(comma+1, data2.length());
      // Serial.println(index);
      // Serial.println(val);

      //update user var
      UserInts[index.toInt()]->set(val.toInt());

    }

    //update a float var
    else if (sub == "flt") {
      // Serial.println("FLOAT");

      //parse index and value
      int comma = data2.lastIndexOf(",");
      String index = data2.substring(4, comma);
      String val = data2.substring(comma+1, data2.length());
      // Serial.println(index);
      // Serial.println(val);

      //update user var
      UserFloats[index.toInt()]->set(val.toFloat());

    }

    //else ignore invalid message

  }
}

//handles a ws event from the ws server
void Dashboard::onEvent(AsyncWebSocket *server, AsyncWebSocketClient *client, AwsEventType type, void *arg, uint8_t *data, size_t len) {
  switch (type) {
    case WS_EVT_CONNECT:
      Serial.printf("WebSocket client #%u connected from %s\n", client->id(), client->remoteIP().toString().c_str());
      this->clients++;
      break;
    case WS_EVT_DISCONNECT:
      Serial.printf("WebSocket client #%u disconnected\n", client->id());
      this->clients--;
      break;
    case WS_EVT_DATA:
      handleWebSocketMessage(arg, data, len);
      break;
    case WS_EVT_PONG:
    case WS_EVT_ERROR:
      break;
  }
}

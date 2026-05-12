/*
Author: Django Wardlow

Script file that is responsible for reading data from the Rose-Hulman car and displaying it on the graph
*/


let sample = 0;

//37 slots, 7 pwr + 16 user ints + 16 user floats
graph_data = [
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
  [],
];

let template_float_series = {
  // initial toggled state (optional)
  show: false,

  spanGaps: false,

  // in-legend display
  label: "!!BAD!!",
  value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(5),

  // series style
  stroke: "red",
  width: 1,
  // fill: "rgba(255, 0, 0, 0.3)",
  // dash: [10, 5],
};

let opts = {
  title: "Car data",
  id: "chart1",
  class: "my-chart",
  width: window.innerWidth - 20,
  height: window.innerHeight - 250,
  scales: {
    x: {
      time: false,
    }
  },
  series: [
    {},
    {
      // initial toggled state (optional)
      show: true,
  
      spanGaps: false,
  
      // in-legend display
      label: "Voltage",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "V",
  
      // series style
      stroke: "green",
      width: 1,
      // fill: "rgba(0, 255, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: true,
  
      spanGaps: false,
  
      // in-legend display
      label: "Current",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(3) + "A",
  
      // series style
      stroke: "red",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "Power",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(3) + "W",
  
      // series style
      stroke: "orange",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
        {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "Energy available",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "J",
  
      // series style
      stroke: "purple",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "Energy used",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "J",
  
      // series style
      stroke: "blue",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "Energy charged",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "J",
  
      // series style
      stroke: "grey",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    }
  ],
};

let uplot = new uPlot(opts, graph_data, document.getElementById("chart-data"));


function clear_graph(){

  sample = 0;

  graph_data = [
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
  ];

  uplot.setData(graph_data);

}


//update the power data on the dashboard
function update_power_data(pwr_data){

  let checked = document.getElementById("run-graph").checked;

  if (isNumeric(pwr_data.voltage) && checked){

    sample += 1;

    graph_data[0].push(sample);
    graph_data[1].push(parseFloat(pwr_data.voltage));
    graph_data[2].push(parseFloat(pwr_data.current)/1000);
    graph_data[3].push(parseFloat(pwr_data.watts)/1000);
    graph_data[4].push(parseFloat(pwr_data.available));
    graph_data[5].push(pwr_data.energy/1000);
    graph_data[6].push(parseFloat(pwr_data.charged));

    uplot.setData(graph_data);

  }

}

function isNumeric(str) {
  if (typeof str != "string") return false // we only process strings!  
  return !isNaN(str) && // use type coercion to parse the _entirety_ of the string (`parseFloat` alone does not do this)...
         !isNaN(parseFloat(str)) // ...and ensure strings of whitespace fail
}


//list of all user ints
user_ints = [];

user_ints_length = 0;

user_int_start = 7;

function update_user_ints(data){

  //parse ints into array
  new_user_ints = [];

  updates = data.split(":");

  for (i = 0; i < updates.length; i++){
    vals = updates[i].split(",");

    user_int = {};

    user_int.name = vals[1];

    //convert string to int
    user_int.val = Math.round(vals[2]);

    // user_int.focused = false;

    new_user_ints[Math.round(vals[0])] = user_int;


  }

  //if any of the ints change rebuild the list

  changed = false;

  //detect if the number of ints changed
  if(new_user_ints.length != user_ints.length){
    changed = true;
  }

  //detect if the names changed
  else{
    for(i = 0; i < new_user_ints.length; i++){
      if(new_user_ints[i].name != user_ints[i].name){
        changed = true;
      }
    }
  }

  if (changed){

    let start_hsl = 0;
    let end_hsl = 180;

    let hsl_inc = (end_hsl-start_hsl)/new_user_ints.length;

    for(let i = 0; i < user_ints_length; i++){
      uplot.delSeries(i + user_int_start);
    }

    clear_graph();

    user_ints_length = new_user_ints.length;


    uplot.setData(graph_data);

    // Loop to create each input item
    for (let i = 0; i < new_user_ints.length; i++) {

      let color = "hsla(" + i*hsl_inc + " 100 50)";

      let series = {
        // initial toggled state (optional)
        show: false,
      
        spanGaps: false,
      
        // in-legend display
        label: new_user_ints[i].name,
        value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(0),
      
        // series style
        stroke: color,
        width: 1,
        // fill: "rgba(255, 0, 0, 0.3)",
        // dash: [10, 5],
      };


      uplot.addSeries(series, user_int_start+i);

    }

  }

  //update the values of existing ints

  for (let i = 0; i < new_user_ints.length; i++) {

    graph_data[user_int_start + i].push(new_user_ints[i].val);

  }


  user_ints = new_user_ints;

}


//list of all user ints
user_floats = [];

user_float = [];

user_floats_length = 0;

function update_user_floats(data){

  user_float_start = user_int_start + user_ints_length;

  //parse ints into array
  new_user_floats = [];

  updates = data.split(":");

  for (i = 0; i < updates.length; i++){
    vals = updates[i].split(",");

    user_float = {};

    user_float.name = vals[1];

    //convert string to num
    user_float.val = Number(vals[2]);

    // user_int.focused = false;

    new_user_floats[Math.round(vals[0])] = user_float;


  }

  //if any of the ints change rebuild the list

  changed = false;

  //detect if the number of ints changed
  if(new_user_floats.length != user_floats.length){
    changed = true;
  }

  //detect if the names changed
  else{
    for(i = 0; i < new_user_floats.length; i++){
      if(new_user_floats[i].name != user_floats[i].name){
        changed = true;
      }
    }
  }

  if (changed){

    
    let start_hsl = 180;
    let end_hsl = 360;

    let hsl_inc = (end_hsl-start_hsl)/new_user_floats.length;

    for(let i = 0; i < user_floats_length; i++){
      try{
        uplot.delSeries(i + user_float_start);
      }
      catch (error){
        console.error(error);
      }
      
    }

    clear_graph();

    user_floats_length = new_user_floats.length;

    uplot.setData(graph_data);

    // Loop to create each input item
    for (let i = 0; i < new_user_floats.length; i++) {
      
      let series = {
        // initial toggled state (optional)
        show: false,
      
        spanGaps: false,
      
        // in-legend display
        label: new_user_floats[i].name,
        value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(5),
      
        // series style
        stroke: "hsla(" + i*hsl_inc + " 100 50)",
        width: 1,
        // fill: "rgba(255, 0, 0, 0.3)",
        // dash: [10, 5],
      };


      uplot.addSeries(series, user_float_start+i);     

    }

  }

  //update the values of existing ints

  for (let i = 0; i < new_user_floats.length; i++) {

    graph_data[user_float_start + i].push(new_user_floats[i].val);

  }

  user_floats = new_user_floats;

}



//Sets up the listners for the Server Side Events
if (!!window.EventSource) {
  var source = new EventSource('/events');
  
  source.addEventListener('open', function(e) {
    console.log("Events Connected");
  }, false);

  source.addEventListener('error', function(e) {
    if (e.target.readyState != EventSource.OPEN) {
      console.log("Events Disconnected");
    }
  }, false);
  
  source.addEventListener('message', function(e) {
    console.log("message", e.data);
  }, false);


  //get pwr data json and decode it
  source.addEventListener('pwr_data', function(e) {
    // console.log("pwr_data_raw", e.data);
    var msg_json = JSON.parse(e.data);

    update_power_data(msg_json);

  }, false);

  //get user int updates
  source.addEventListener('user_ints', function(e) {
    //console.log("user_ints:", e.data);

    update_user_ints(e.data);

  }, false);

  //get pwr data json and decode it
  source.addEventListener('user_floats', function(e) {
    //console.log("user_floats:", e.data);

    update_user_floats(e.data);

  }, false);
}

//web sockets used for race buttons and eventually user vars
// https://randomnerdtutorials.com/esp32-websocket-server-arduino/
var gateway = `ws://${window.location.hostname}/ws`;
var websocket;
function initWebSocket() {
  console.log('Trying to open a WebSocket connection...');
  websocket = new WebSocket(gateway);
  websocket.onopen    = onOpen;
  websocket.onclose   = onClose;
  websocket.onmessage = onMessage;
}
function onOpen(event) {
  console.log('Connection opened');
}

//attempt to reopen WS connection if closed
function onClose(event) {
  console.log('Connection closed');
  setTimeout(initWebSocket, 2000);
}

//handles messages for the websocket
function onMessage(event) {
  //console.log("WS event: ", event.data);

  // const timer = document.getElementById("timer");

  // timer.textContent = event.data + " S";
}

//init websocket on page load
window.addEventListener('load', onLoad);

function onLoad(event) {
  console.log("LOAD");
  initWebSocket();
}
/*
Author: Django Wardlow

Script file that is responsible for controlling the power supply
*/


let sample = 0;

//data slots
graph_data = [
  [], //sample
  [], //V out 
  [], //I out
  [], //P out
  [], //V in
  [], //I in
  [], //P in
  [], //efficacy
  [], //duty
  [], //load R
];

let opts = {
  title: "power supply data",
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
      label: "V out",
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
      label: "I out",
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
      label: "P out",
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
      label: "V in",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "V",
  
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
      label: "I in",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "A",
  
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
      label: "P in",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "W",
  
      // series style
      stroke: "grey",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "efficacy",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "%",
  
      // series style
      stroke: "grey",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "duty",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "%",
  
      // series style
      stroke: "grey",
      width: 1,
      // fill: "rgba(255, 0, 0, 0.3)",
      // dash: [10, 5],
    },
    {
      // initial toggled state (optional)
      show: false,
  
      spanGaps: false,
  
      // in-legend display
      label: "load R",
      value: (self, rawValue) => rawValue == null ? '' : "" + rawValue.toFixed(2) + "Ω",
  
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
    [], //sample
    [], //V out 
    [], //I out
    [], //P out
    [], //V in
    [], //I in
    [], //P in
    [], //efficacy
    [], //duty
    [], //load R
  ];

  uplot.setData(graph_data);

}


function isNumeric(str) {
  if (typeof str != "string") return false // we only process strings!  
  return !isNaN(str) && // use type coercion to parse the _entirety_ of the string (`parseFloat` alone does not do this)...
         !isNaN(parseFloat(str)) // ...and ensure strings of whitespace fail
}



//update the power data on the dashboard
function update_power_data(data){

  let checked = document.getElementById("run-graph").checked;

  if (isNumeric(pwr_data.voltage) && checked){

    sample += 1;

    graph_data[0].push(sample);
    graph_data[1].push(parseFloat(pwr_data.vout));
    graph_data[2].push(parseFloat(pwr_data.iout)/1000);
    graph_data[3].push(parseFloat(pwr_data.pout)/1000);
    graph_data[4].push(parseFloat(pwr_data.vin));
    graph_data[5].push(parseFloat(pwr_data.iin)/1000);
    graph_data[6].push(parseFloat(pwr_data.pin)/1000);
    graph_data[7].push(parseFloat(pwr_data.eff));
    graph_data[8].push(parseFloat(pwr_data.duty));
    graph_data[9].push(parseFloat(pwr_data.rload));


    uplot.setData(graph_data);

  }

  const voltage = document.getElementById("Vout");

  voltage.textContent = data.vout + " V";

  const current = document.getElementById("Iout");

  current.textContent = data.iout + " mA";

  const power = document.getElementById("Pout");

  power.textContent = data.pout + " J";

  const watts = document.getElementById("watts");

  watts.textContent = data.watts + " mW";

  const volt_in = document.getElementById("Vin");

  energy.textContent = (data.vin) + " V";

  const eff = document.getElementById("Efficiency");

  eff.textContent = (data.eff) + " η";

  const duty = document.getElementById("Duty");

  duty.textContent = (data.duty) + " %";

}

//sends a new value back to the power supply
function value_changed(element){
  val = element.value;
  name = element.id;
  console.log("setting change", name, val);
  websocket.send(name+":"+val);
}


//web sockets connection setup
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
  console.log("WS event: ", event.data);

  var msg_json = JSON.parse(event.data);

  update_power_data(msg_json);

}

//init websocket on page load
window.addEventListener('load', onLoad);

function onLoad(event) {
  console.log("LOAD");
  initWebSocket();
}


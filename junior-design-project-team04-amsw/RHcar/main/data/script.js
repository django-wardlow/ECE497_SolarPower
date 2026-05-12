/*
Author: Django Wardlow

Script file that is responsible for reading data from the Rose-Hulman car and displaying it on the dashboard
*/

//event server based on https://randomnerdtutorials.com/esp32-web-server-gauges/

//adds log entry from the car to the log area on the dashboard
function add_log_entry(msg){

  console.log(msg);

  // Create a new <p> element
  const newParagraph = document.createElement("p");

  // Create a new text node
  const newText = document.createTextNode(msg);

  // Append the text to the <p> element
  newParagraph.appendChild(newText);

  // Find the container where text will be added
  const container = document.getElementById("text-container");

  // prepend the new <p> element to the container
  container.prepend(newParagraph);
}

//update the power data on the dashboard
function update_power_data(data){

  const voltage = document.getElementById("voltage");

  voltage.textContent = data.voltage + " V";

  const current = document.getElementById("current");

  current.textContent = data.current + " mA";

  const available = document.getElementById("available");

  available.textContent = data.available + " J";

  const watts = document.getElementById("watts");

  watts.textContent = data.watts + " mW";

  const energy = document.getElementById("race-energy");

  energy.textContent = (data.energy/1000).toFixed(4) + " J used";

  const c_energy = document.getElementById("charge-energy");

  c_energy.textContent = (data.charged) + " J charged";

}

function user_int_changed(index, newval){
  console.log("int change", index, newval);
  websocket.send("int,"+index+","+newval);
}


//list of all user ints
user_ints = [];

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

    //remove existing entrys

    let old_inputs = document.getElementById("userint_container");

    if(old_inputs){
      old_inputs.remove();
    }


    container = document.createElement('div');
    container.id = "userint_container";

    // Loop to create each input item
    for (let i = 0; i < new_user_ints.length; i++) {
      // --- Create the elements for each item ---

      // 1. Create a container div for the label and input for layout
      const itemDiv = document.createElement('div');
      itemDiv.className = 'userinputbox'; 
      itemDiv.id = "userint"

      // 2. Create the label element (the "title")
      const label = document.createElement('label');
      const inputId = 'userint' + i; // Create a unique ID for the input
      label.setAttribute('for', inputId); // Link the label to the input using 'for'
      label.textContent = new_user_ints[i].name;    // Set the text content of the label

      label.className = 'userinputlabel'; // Fixed width for alignment

      // 3. Create the number input element
      const input = document.createElement('input');
      input.setAttribute('type', 'number');
      input.setAttribute('id', inputId);    // Set the unique ID
      input.setAttribute('name', 'itemValue' + i); // Optional: Set a name if using in a form
      input.setAttribute('placeholder', '0'); // Optional: Add a placeholder
      // input.setAttribute('value', '0'); // Optional: Set a default value
      // input.setAttribute('min', '0'); // Optional: Set minimum value
      // input.setAttribute('max', '100'); // Optional: Set maximum value

      //detect when user is editing the input
      // input.addEventListener('focus', () => {
      //   new_user_ints[i].focused = true;
      //   console.log('Editing started');
      // });
      
      // input.addEventListener('blur', () => {
      //   new_user_ints[i].focused = false;
      //   console.log('Editing ended');
      // });

      input.addEventListener('change', () => {
        user_int_changed(i, input.value);
      });

      input.className = 'userinput';

      // --- Assemble the item ---
      itemDiv.appendChild(label); // Add the label to the item's div
      itemDiv.appendChild(input); // Add the input to the item's div

      // --- Add the complete item to the main container ---
      container.appendChild(itemDiv);

    }

    parent = document.getElementById("u_ints");

    parent.appendChild(container);

  }

  //update the values of existing ints

  for (let i = 0; i < new_user_ints.length; i++) {

    input = document.getElementById('userint' + i);

    //dont overwright value if the user is typing into the box
    if(document.activeElement != input){
      input.value = new_user_ints[i].val;
    }

  }


  user_ints = new_user_ints;

}


function user_float_changed(index, newval){
  console.log("float change", index, newval);
  websocket.send("flt,"+index+","+newval);
}


//list of all user ints
user_floats = [];

function update_user_floats(data){

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

    //remove existing entrys

    let old_inputs = document.getElementById("userfloat_container");

    if(old_inputs){
      old_inputs.remove();
    }


    container = document.createElement('div');
    container.id = "userfloat_container";

    // Loop to create each input item
    for (let i = 0; i < new_user_floats.length; i++) {
      // --- Create the elements for each item ---

      // 1. Create a container div for the label and input for layout
      const itemDiv = document.createElement('div');
      itemDiv.className = 'userinputbox'; 
      itemDiv.id = "userfloat"

      // 2. Create the label element (the "title")
      const label = document.createElement('label');
      const inputId = 'userfloat' + i; // Create a unique ID for the input
      label.setAttribute('for', inputId); // Link the label to the input using 'for'
      label.textContent = new_user_floats[i].name;    // Set the text content of the label

      label.className = 'userinputlabel'; // Fixed width for alignment

      // 3. Create the number input element
      const input = document.createElement('input');
      input.setAttribute('type', 'number');
      input.setAttribute('step', '0.00001');
      input.setAttribute('id', inputId);    // Set the unique ID
      input.setAttribute('name', 'itemValue' + i); // Optional: Set a name if using in a form
      input.setAttribute('placeholder', '0'); // Optional: Add a placeholder
      // input.setAttribute('value', '0'); // Optional: Set a default value
      // input.setAttribute('min', '0'); // Optional: Set minimum value
      // input.setAttribute('max', '100'); // Optional: Set maximum value

      //detect when user is editing the input
      // input.addEventListener('focus', () => {
      //   new_user_ints[i].focused = true;
      //   console.log('Editing started');
      // });
      
      // input.addEventListener('blur', () => {
      //   new_user_ints[i].focused = false;
      //   console.log('Editing ended');
      // });

      input.addEventListener('change', () => {
        user_float_changed(i, input.value);
      });

      input.className = 'userinput';

      // --- Assemble the item ---
      itemDiv.appendChild(label); // Add the label to the item's div
      itemDiv.appendChild(input); // Add the input to the item's div

      // --- Add the complete item to the main container ---
      container.appendChild(itemDiv);

    }

    parent = document.getElementById("u_floats");

    parent.appendChild(container);

  }

  //update the values of existing ints

  for (let i = 0; i < new_user_floats.length; i++) {

    input = document.getElementById('userfloat' + i);

    //dont overwright value if the user is typing into the box
    if(document.activeElement != input){
      input.value = new_user_floats[i].val;
    }

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
  
  //get raw text and send it to add log fn
  source.addEventListener('log_message', function(e) {
    console.log("new_log_msg_raw", e.data);

    add_log_entry(e.data);

  }, false);

  //get pwr data json and decode it
  source.addEventListener('pwr_data', function(e) {
    // console.log("pwr_data_raw", e.data);
    var msg_json = JSON.parse(e.data);

    update_power_data(msg_json);

  }, false);

  //get pwr data json and decode it
  source.addEventListener('user_ints', function(e) {
    //console.log("user_ints:", e.data);

    update_user_ints(e.data);

  }, false);

  //get pwr data json and decode it
  source.addEventListener('user_floats', function(e) {
    //console.log("user_floats:", e.data);

    update_user_floats(e.data);
    // var msg_json = JSON.parse(e.data);

    // update_power_data(msg_json);

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

  const timer = document.getElementById("timer");

  timer.textContent = event.data + " S";
}

//init websocket on page load
window.addEventListener('load', onLoad);

function onLoad(event) {
  console.log("LOAD");
  initWebSocket();
}

//race button functions
function start_race_btn(){
  websocket.send('start');
}

function manual_charge_btn(){
  websocket.send('charge');
}

function stop_race_btn(){
  websocket.send('stop');
}

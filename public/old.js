
// atob is the browser built in base64 decoder
function decodeBase64Text(base) {
    try {
        return atob(base);
    } catch (e) {
        console.log("Error decoding Base64", e);
        return null;
    }
}

// add our hello world
const a = document.createElement("div");
a.innerHTML = "dude...";
document.body.appendChild(a);

let b = document.getElementById("bin-text");
const c = b.innerHTML;
const d = document.createElement("div");
d.innerHTML = decodeBase64Text(c);
if (d) {
    document.body.appendChild(d);
}

// add a picture
const img = document.createElement("img");
const base64DataPNG = document.getElementById("bin-png").innerHTML.trim();
img.src = `data:image/png;base64,${base64DataPNG}`;
img.width = 368;
img.height = 547;
if (img) {
    document.body.appendChild(img);
}

// Modified version of the wasm-bindgen generated code
const wasmBinary = atob(document.getElementById('bin-wasm').innerHTML.trim());
const wasmBytes = new Uint8Array(wasmBinary.length);
for (let i = 0; i < wasmBinary.length; i++) {
  wasmBytes[i] = wasmBinary.charCodeAt(i);
}

// Override the fetch function used by wasm-bindgen
const originalFetch = window.fetch;
window.fetch = function(url) {
  if (url.endsWith('.wasm')) {
    return Promise.resolve(new Response(wasmBytes.buffer, {
      status: 200,
      headers: { 'Content-Type': 'application/wasm' }
    }));
  }
  return originalFetch(url);
};

// The rest of your wasm-bindgen generated code can remain mostly unchanged
// ...
//
async function runApp() {
  // Initialize the WASM module (function name depends on your crate name)
  // If your crate is named "my_wasm" it might be:
  await wasm_bindgen();

  // Now call your greet function
  const greeting = my_wasm.greet();
  console.log(greeting); // "Hello from webassembly!"
  
  // Display it on the page
  const greetingElement = document.createElement("div");
  greetingElement.innerHTML = greeting;
  document.body.appendChild(greetingElement);
}

window.addEventListener('DOMContentLoaded', runApp);


/*
// Simple WebAssembly loading function (no wasm-bindgen dependencies)
async function loadWasmModule(base64Wasm) {
    try {
        // Decode base64
        const binary = atob(base64Wasm);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
        }
        
        // Use simple instantiation with empty imports
        const result = await WebAssembly.instantiate(bytes, {});
        console.log("WASM instantiated successfully!");
        return result.instance.exports;
    } catch (error) {
        console.error("Error loading WebAssembly:", error);
        throw error;
    }
}

async function test() {
    try {

        const wasm = document.getElementById('bin-wasm').innerHTML.trim();
        console.log("Loading WebAssembly module...");
        
        const wasmExports = await loadWasmModule(wasm);
        console.log("Available WebAssembly exports:", Object.keys(wasmExports));
        
        // List all available functions
        for (const key of Object.keys(wasmExports)) {
            if (typeof wasmExports[key] === 'function') {
                console.log(`Found WebAssembly function: ${key}`);
            }
        }
        
        // Use the add function if it exists
        if (typeof wasmExports.add === 'function') {
            const sum = wasmExports.add(5, 7);
            console.log("5 + 7 =", sum);
            
            // Display on the page
            const mathResult = document.createElement("div");
            mathResult.innerHTML = `<p>WebAssembly calculation: 5 + 7 = ${sum}</p>`;
            document.body.appendChild(mathResult);
        }
        
        // Use the greeting code if it exists
        if (typeof wasmExports.greeting_code === 'function') {
            const code = wasmExports.greeting_code();
            const greeting = `WebAssembly greeting code: ${code}`;
            console.log(greeting);
            
            // Map code to message (just for demo)
            let message = "Hello from WebAssembly!";
            if (code === 42) {
                message = "Hello from WebAssembly! (The answer is 42)";
            }
            
            // Display on the page
            const greetingElement = document.createElement("div");
            greetingElement.innerHTML = `<p>${message}</p>`;
            document.body.appendChild(greetingElement);
        }

        if (typeof wasmExports.greet === 'function') {
            const greet = new TextDecoder().decode(wasmExports.greet());
            const greetingElement = document.createElement("div");
            greetingElement.innerHTML = `<p>${greet}</p>`;
            document.body.appendChild(greetingElement);
        }
        
    } catch (error) {
        console.error("Error in test function:", error);
        
        // Show error on page
        const errorElement = document.createElement("div");
        errorElement.style.color = "red";
        errorElement.innerHTML = `<p>WebAssembly error: ${error.message}</p>`;
        document.body.appendChild(errorElement);
    }
}

// Run the test when the page is loaded
window.addEventListener('DOMContentLoaded', () => {
    test();
});

*/

/*
// atob is the browser built in base64 decoder
function decodeBase64Text(base) {
    try {
        return atob(base);
    } catch (e) {
        console.log("Error decoding Base64", error);
        return null;
    }
}

function decodeBase64PNG(base) {
    try {
        return (base);
    } catch (e) {
        console.log("Error decoding Base64", error);
        return null;
    }
}

// add out hello world
const a = document.createElement("div");
a.innerHTML = "dude...";
document.body.appendChild(a);

let b = document.getElementById("bin-text");
const c = b.innerHTML;

const d = document.createElement("div");
d.innerHTML = decodeBase64Text(c);
if (d) {
    document.body.appendChild(d);
}

// add a picture
const img = document.createElement("img");
const base64DataPNG = document.getElementById("bin-png").innerHTML.trim();
img.src = `data:image/png;base64,${base64DataPNG}`;
img.width = 368;
img.height = 547;
if (img) {
    document.body.appendChild(img);
}

// wasm test
// public/wasm-loader.js
async function loadWasmModule(base64Wasm) {
    const binary = atob(base64Wasm);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    
    const result = await WebAssembly.instantiate(bytes, {});
    return result.instance.exports;
}



async function test() {
    const wasm = document.getElementById('bin-wasm').innerHTML.trim();
    console.log(wasm);
    const imports = {
        wbg: {},    //webassembly bindings generated namespace
        env: {}     // environment imports
    };

    const module = await loadWasmModule(wasm, imports);
    console.log(module);
    console.log(module.greet());
}

test();

*/

//const wasmBase64 = '{}';
//document.addEventListener('DOMContentLoaded', async () => {{
//const wasmExports = await loadWasmModule(wasmBase64);
//    document.getElementById('wasm-output').textContent = wasmExports.hello();
//}});
//
//

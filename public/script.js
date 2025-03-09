// Helper function to decode base64 text
function decodeBase64Text(base64) {
    try {
        return atob(base64);
    } catch (e) {
        console.error("Error decoding Base64", e);
        return null;
    }
}

// Helper function to convert base64 to ArrayBuffer
function base64ToArrayBuffer(base64) {
    // Remove any whitespace that might have been added
    base64 = base64.replace(/\s/g, '');
    
    // Standard base64 decoder
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
}

// Function to add basic content elements
function addBasicContent() {
    // Add welcome message
    const welcome = document.createElement("div");
    welcome.innerHTML = "WebAssembly Embedded Demo";
    welcome.style.fontSize = "24px";
    welcome.style.fontWeight = "bold";
    welcome.style.margin = "20px 0";
    document.body.appendChild(welcome);

    // Add text content from base64
    if (document.getElementById("bin-text")) {
        const textBase64 = document.getElementById("bin-text").innerHTML.trim();
        const textContent = document.createElement("div");
        textContent.innerHTML = decodeBase64Text(textBase64);
        document.body.appendChild(textContent);
    }

    // Add image from base64
    if (document.getElementById("bin-png")) {
        const base64DataPNG = document.getElementById("bin-png").innerHTML.trim();
        const img = document.createElement("img");
        img.src = `data:image/png;base64,${base64DataPNG}`;
        img.width = 368;
        img.height = 547;
        document.body.appendChild(img);
    }
}

// Load and set up WebAssembly module
async function setupWasm() {
    try {
        // Get the base64 encoded WASM
        const wasmBase64 = document.getElementById('bin-wasm').innerHTML.trim();
        
        // Convert to binary
        const wasmBytes = base64ToArrayBuffer(wasmBase64);
        
        // Debug: Check the WebAssembly binary header
        console.log("WASM binary first 8 bytes:", 
            Array.from(wasmBytes.slice(0, 8)).map(b => 
                "0x" + b.toString(16).padStart(2, '0')).join(' '));
        
        // Check for valid WebAssembly magic number
        if (wasmBytes.length < 4 || 
            wasmBytes[0] !== 0x00 || 
            wasmBytes[1] !== 0x61 || 
            wasmBytes[2] !== 0x73 || 
            wasmBytes[3] !== 0x6D) {
            throw new Error("Invalid WebAssembly binary (wrong magic number)");
        }

        // Initialize the WebAssembly module using wasm_bindgen
        // We pass the bytes directly instead of using fetch
        const result = await wasm_bindgen(wasmBytes);
        
        console.log("WASM module initialized successfully!");
        return result;
        
    } catch (error) {
        console.error("WebAssembly setup error:", error);
        
        // Show error on page
        const errorElement = document.createElement("div");
        errorElement.style.color = "red";
        errorElement.style.padding = "10px";
        errorElement.style.margin = "10px 0";
        errorElement.style.border = "1px solid red";
        errorElement.innerHTML = `<p>WebAssembly error: ${error.message}</p>`;
        document.body.appendChild(errorElement);
        
        throw error;
    }
}

// Main application function
async function runApp() {
    try {
        // Add basic content first
        addBasicContent();
        
        // Set up and initialize WebAssembly
        await setupWasm();
        
        // Add a separator
        const separator = document.createElement("hr");
        document.body.appendChild(separator);
        
        // Create a container for WebAssembly output
        const wasmContainer = document.createElement("div");
        wasmContainer.style.padding = "20px";
        wasmContainer.style.margin = "20px 0";
        wasmContainer.style.backgroundColor = "#f5f5f5";
        wasmContainer.style.borderRadius = "5px";
        
        // Title for WebAssembly section
        const wasmTitle = document.createElement("h2");
        wasmTitle.textContent = "WebAssembly Output";
        wasmContainer.appendChild(wasmTitle);
        
        // Use the WebAssembly functions
        try {
            // Call the greet function (exported by wasm_bindgen)
            const greeting = wasm_bindgen.greet();
            console.log("WebAssembly greeting:", greeting);
            
            const greetingElement = document.createElement("div");
            greetingElement.innerHTML = `<p>Message from WebAssembly: <strong>${greeting}</strong></p>`;
            wasmContainer.appendChild(greetingElement);

            // testing new
            const hello = wasm_bindgen.hello();
            const bye = wasm_bindgen.goodbye();
            const ele = document.createElement("div");

            ele.innerHTML = `<pre>You say, "Yes", I say, "No"
You say, "Stop" and I say, "Go, go, go"
Oh no
You say, "${bye}" and I say, "${hello}, ${hello}, ${hello}"
I don't know why you say, "${bye}", I say, "${hello}, ${hello}, ${hello}"
I don't know why you say, "${bye}", I say, "${hello}"
</pre>`;
            wasmContainer.appendChild(ele);
            
        } catch (e) {
            console.error("Error calling WebAssembly function:", e);
            const errorMsg = document.createElement("p");
            errorMsg.style.color = "red";
            errorMsg.textContent = `Error calling WebAssembly function: ${e.message}`;
            wasmContainer.appendChild(errorMsg);
        }
        
        // Add the WebAssembly container to the page
        document.body.appendChild(wasmContainer);
        
    } catch (error) {
        console.error("Application error:", error);
    }
}

// Run the application when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);

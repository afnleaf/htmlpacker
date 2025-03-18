// load base64 from html
// set up WASM module
async function setupWasm() {
    try {
        console.log("Setting up Bevy WebAssembly application...");
        
        // Get the base64 encoded WASM
        const wasmBase64 = document.getElementById('bin-wasm').innerHTML.trim();
        // Decode to binary
        const wasmBytes = base64ToArrayBuffer(wasmBase64);
        
        // Check WASM binary header for validity
        const magicBytes = Array.from(wasmBytes.slice(0, 4));
        console.log("WASM binary magic bytes:", 
            magicBytes.map(b => "0x" + b.toString(16).padStart(2, '0')).join(' '));
        
        // Check for valid WASM magic number (0x00 0x61 0x73 0x6D)
        if (wasmBytes.length < 4 || 
            wasmBytes[0] !== 0x00 || 
            wasmBytes[1] !== 0x61 || 
            wasmBytes[2] !== 0x73 || 
            wasmBytes[3] !== 0x6D) {
            throw new Error("Invalid WASM binary (wrong magic number)");
        }
        
        // Initialize WASM module using wasm_bindgen script
        console.log("Initializing Bevy engine...");
        await wasm_bindgen(wasmBytes);
        
        console.log("Bevy engine initialized successfully!");
        
    } catch (error) {
        console.error("Error initializing Bevy:", error);
        
        // Show error on page
        const errorElement = document.createElement("div");
        errorElement.style.position = "absolute";
        errorElement.style.top = "10px";
        errorElement.style.left = "10px";
        errorElement.style.zIndex = "1000";
        errorElement.style.color = "red";
        errorElement.style.backgroundColor = "rgba(0,0,0,0.7)";
        errorElement.style.padding = "10px";
        errorElement.style.borderRadius = "5px";
        errorElement.style.fontFamily = "monospace";
        errorElement.innerHTML = `<p>Bevy initialization error: ${error.message}</p>`;
        document.body.appendChild(errorElement);
        
        throw error;
    }
}

// convert base64 to ArrayBuffer u8 bytes
function base64ToArrayBuffer(base64) {
    // remove any whitespace that might have been introduced in the HTML
    base64 = base64.replace(/\s/g, '');
    
    // decode base64 to binary string
    const binaryString = atob(base64);
    
    // convert binary string to Uint8Array
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    
    return bytes;
}

// main app function
async function runApp() {
    console.log("Starting Bevy application...");
    try {
        await setupWasm();
    } catch (error) {
        console.error("Fatal error starting Bevy application:", error);
    }
}

// run app when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);

// load base64 from html
// set up WASM module
// bound to the name, wasm_bindgen
async function setupWasm() {
    try {
        console.log("Setting up WASM application...");

        // get the base64 encoded WASM
        const wasmBase64 = document.getElementById('bin-wasm').innerHTML.trim();
        // decode to binary
        //const wasmBytes = base64ToArrayBuffer(wasmBase64);
        const compressedBytes = base64ToArrayBuffer(wasmBase64);
        const wasmBytes = await decompressBrotli(compressedBytes); 

        // check WASM binary header for validity
        // first 4 bytes
        const magicBytes = Array.from(wasmBytes.slice(0, 4));
        console.log("WASM binary magic bytes:", 
            magicBytes.map(b => "0x" + b.toString(16).padStart(2, '0')).join(' '));
        
        // check for valid WASM magic number
        if (wasmBytes.length < 4 || 
            wasmBytes[0] !== 0x00 || 
            wasmBytes[1] !== 0x61 || 
            wasmBytes[2] !== 0x73 || 
            wasmBytes[3] !== 0x6D) {
            throw new Error("Invalid WASM binary (wrong header)");
        }

        // init WASM module using wasm_bindgen script
        // pass in bytes directly instead of using fetch
        //const result = await wasm_bindgen(wasmBytes);
        await wasm_bindgen(wasmBytes);
        // this never gets printed because an error is thrown from bevy
        // its a fake error though, how do we get around it
        // probably catch it in bevy!
        console.log("WASM module initialized successfully!");
        //return result;
        
    } catch (error) {
        console.error("WASM setup error:", error);
        
        // show error on page
        // we can style this better
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
        errorElement.innerHTML = `<p>WASM initialization error: ${error.message}</p>`;
        document.body.appendChild(errorElement);
        
        throw error;

    }
}

// catch error in decode
function decodeBase64Text(base64) {
    try {
        return atob(base64);
    } catch (e) {
        console.error("Error decoding Base64", e);
        return null;
    }
}

// convert base64 to ArrayBuffer u8 bytes
function base64ToArrayBuffer(base64) {
    // remove any whitespace
    base64 = base64.replace(/\s/g, '');
    //
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
}

/**
 * Decompress Brotli data using native browser APIs when available,
 * or fall back to BrotliDecode library
 */
async function decompressBrotli(compressedData) {
    // Method 1: Use native browser DecompressionStream if available (modern browsers)
    if (typeof DecompressionStream === 'function') {
        try {
            console.log("Using native DecompressionStream API for Brotli decompression");
            const stream = new Response(compressedData).body
                .pipeThrough(new DecompressionStream('br'));
            
            return new Uint8Array(await new Response(stream).arrayBuffer());
        } catch (e) {
            console.warn("Native DecompressionStream failed:", e);
            // Fall back to method 2
        }
    }
    
    // Method 2: Use a JS Brotli decoder library (BrotliDecode)
    // This requires including the BrotliDecode.js library in your page
    if (typeof BrotliDecode === 'function') {
        console.log("Using BrotliDecode.js for Brotli decompression");
        return BrotliDecode(compressedData);
    }
    
    // If no method is available, throw an error
    throw new Error("No Brotli decompression method available. Please include BrotliDecode.js or use a browser with DecompressionStream support.");
}

// You'll need to include the BrotliDecode.js library in your page for fallback support
// Add this to your HTML:
// <script src="https://cdnjs.cloudflare.com/ajax/libs/brotli/1.3.2/decode.min.js"></script>

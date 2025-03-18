// load base64 from html
// set up WASM module
async function setupWasm() {
    try {
        // get the base64 encoded WASM
        const wasmBase64 = document.getElementById('bin-wasm').innerHTML.trim();
        // decode to binary
        const wasmBytes = base64ToArrayBuffer(wasmBase64);
        
        // check WASM binary header for validity
        console.log("WASM binary first 8 bytes:", 
            Array.from(wasmBytes.slice(0, 8)).map(b => 
                "0x" + b.toString(16).padStart(2, '0')).join(' '));
        // check for valid WASM magic number
        if (wasmBytes.length < 4 || 
            wasmBytes[0] !== 0x00 || 
            wasmBytes[1] !== 0x61 || 
            wasmBytes[2] !== 0x73 || 
            wasmBytes[3] !== 0x6D) {
            throw new Error("Invalid WASM binary (wrong magic number)");
        }

        // init WASM module using wasm_bindgen script
        // pass in bytes directly instead of using fetch
        const result = await wasm_bindgen(wasmBytes);
        
        console.log("WASM module initialized successfully!");
        return result;
        
    } catch (error) {
        console.error("WASM setup error:", error);
        
        // show error on page
        // we can style this better
        const errorElement = document.createElement("div");
        errorElement.style.color = "red";
        errorElement.style.padding = "10px";
        errorElement.style.margin = "10px 0";
        errorElement.style.border = "1px solid red";
        errorElement.innerHTML = `<p>WASM error: ${error.message}</p>`;
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

/*
WARNING
THIS SCRIPT IS OMEGA SLOP
*/

// load base64 from html
// set up WASM module
// bound to the name, wasm_bindgen
async function setupWasm() {
    try {
        console.log("Setting up WASM application...");
        console.log("Loading wasm decoder...");
        // get the base64 encoded WASM
        // decode to binary
        const wasmDecoder64 = document.getElementById('bin-wasm-decoder').innerHTML.trim();
        const wasmBytes = base64ToArrayBuffer(wasmDecoder64);
        checkMagicBytes(wasmBytes);
        // pass in bytes directly instead of using fetch
        // init WASM decoder module using glue script
        await wasm_decoder(wasmBytes);
        
        console.log("Loading wasm main app...");
        // get the main binary
        const wasmMain64 = document.getElementById('bin-wasm').innerHTML.trim();
        const compressedBytes = base64ToArrayBuffer(wasmMain64);
        //const compressedBytes = await wasm_decoder.decodeBase94(wasmBase64mod);
        const wasmBytes2 = await wasm_decoder.decompress(compressedBytes);
        await wasm_bindgen(wasmBytes2);
        console.log("WASM module initialized successfully!");
    
    } catch (error) {
        console.error("WASM setup error:", error);
        appendErrorMessage(error);
        throw error;
    }
}

function appendErrorMessage(error) {
    // show error on page
    // we can style this better
    // this always gets printed because an error is thrown from bevy
    // its a fake error though, how do we get around it
    // probably catch it in bevy!
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
}

// check WASM binary header for validity
// first 4 bytes
function checkMagicBytes(wasmBytes) {
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


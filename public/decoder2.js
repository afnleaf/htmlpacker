/*
* decoder.js
* the point of these functions is to decode the embedded application 
* trying out immediately invoked function expression
*/

(async () => {
    // check WASM binary header for validity
    // first 4 bytes
    function checkMagicBytes(bytes) {
        const magic = Array.from(bytes.slice(0, 4));
        const p = magic.map(b => "0x" + b.toString(16).padStart(2, '0')).join(' ');
        console.log("WASM binary magic bytes:", p);
        // check for valid WASM magic number
        if (bytes.length < 4  || 
            bytes[0] !== 0x00 || 
            bytes[1] !== 0x61 || 
            bytes[2] !== 0x73 || 
            bytes[3] !== 0x6D) {
            throw new Error("Invalid WASM binary (wrong header)");
        }
    }
    
    // convert base64 to ArrayBuffer u8 bytes
    function b64ToBytes(base64) {
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
        
    
    async function loadDecoder() {
        console.log("Loading wasm decoder...");
        // get the base64 encoded WASM, decode to binary
        const wasmDecoder64 = document.getElementById('bin-wasm-decoder').innerHTML.trim();
        const wasmBytes = b64ToBytes(wasmDecoder64);
        checkMagicBytes(wasmBytes);
        // pass in bytes directly instead of using fetch
        // init WASM decoder module using glue script
        // linter will think this is missing because we 
        // renamed wasm_bindgen global var to wasm_decoder
        await wasm_decoder(wasmBytes);
    
    }
    
    async function loadApp() {
        console.log("Loading wasm main app...");
        // get the main binary
        const wasmMain64 = document.getElementById('bin-wasm').innerHTML.trim();
        const compressedBytes = b64ToBytes(wasmMain64);
        const wasmBytes = await wasm_decoder.decompress(compressedBytes);
        //
        await wasm_bindgen(wasmBytes);
    }
    
    // load base64 from html
    // set up WASM module
    // bound to the name, wasm_bindgen
    // only this function exposed globally
    window.setupWasm = async function()  {
        try {
            console.log("Setting up WASM application...");
            await loadDecoder();
            await loadApp();
            console.log("WASM module initialized successfully!");
        
        } catch (error) {
            console.error("WASM setup error:", error);
            throw error;
        }
    }

})();





/*
* decoder.js
* the point of these functions is to decode the embedded application 
* trying out immediately invoked function expression
* adding an indexedDB to cache u8 from b64 + brotli for subsequent loads
* code status = slop, needs a refactor after feature implementation
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
        
    
    async function loadDecoder(db) {
        console.log("Loading wasm decoder...");
        // get the base64 encoded WASM, decode to binary
        const wasmDecoderElement = document.getElementById('bin-wasm-decoder');
        const wasmDecoderHash = wasmDecoderElement.getAttribute('hash');
        let wasmBytes;
        if (!wasmDecoderHash) {
            console.log("No hash found for decoder, can't access indexedDB.");
        } else {
            console.log(wasmDecoderHash);
            // first check if hash is in indexedDB
            const cacheKey = `wasm-decoder-${wasmDecoderHash}`;

            const cachedBytes = await getAssetFromCache(db, cacheKey);

            if (cachedBytes) {
                // on cache hit
                console.log("Decoder module loaded from IndexedDB cache.");
                wasmBytes = cachedBytes;
            } else {
                // on cache miss
                console.log("Decoder module not found in cache. Decoding pre");
                const wasmDecoder64 = wasmDecoderElement.innerHTML.trim();
                wasmBytes = b64ToBytes(wasmDecoder64);
                checkMagicBytes(wasmBytes);
                                
                try {
                    await saveAssetToCache(db, cacheKey, wasmBytes);
                } catch(e) {
                    console.warn("Error: ", e)
                    // not sure if this error should be propagated
                }
            }
        }

        // pass in bytes directly instead of using fetch
        // init WASM decoder module using glue script
        // linter will think this is missing because we 
        // renamed wasm_bindgen global var to wasm_decoder
        await wasm_decoder(wasmBytes);
    
    }

    async function loadApp(db) {
        console.log("Loading wasm main app...");
        let wasmAppElement = document.getElementById('bin-wasm-app')
        const wasmAppHash = wasmAppElement.getAttribute('hash');
        let wasmBytes;

        if (!wasmAppHash) {
            console.log("No hash found for app, can't access indexedDB.");
        } else {
            console.log(wasmAppHash);
            // first check if hash is in indexedDB
            const cacheKey = `wasm-app-${wasmAppHash}`;
            const cachedBytes = await getAssetFromCache(db, cacheKey);

            if (cachedBytes) {
                // on cache hit
                console.log("App module loaded from IndexedDB cache.");
                wasmBytes = cachedBytes;
            } else {
                // on cache miss
                console.log("App module not found in cache. Decoding pre");
                const wasmApp64 = wasmAppElement.innerHTML.trim();
                const b = b64ToBytes(wasmApp64);
                wasmBytes = await wasm_decoder.decompress(b);
                checkMagicBytes(wasmBytes);
                
                try {
                    await saveAssetToCache(db, cacheKey, wasmBytes);
                } catch(e) {
                    console.warn("Error: ", e)
                    // not sure if this error should be propagated
                }
            }
        }

        await wasm_bindgen(wasmBytes);
    }
    
    async function loadAppOld() {
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
            console.log("Loading IndexedDB");
            const db = await openDb();

            console.log("Setting up WASM application...");
            await loadDecoder(db);
            await loadApp(db);
            console.log("WASM module initialized successfully!");
        
        } catch (e) {
            console.error("WASM setup error:", e);
            throw e;
        }
    }

    // these constants are how we open the db
    // version only increments when you change the object stores
    const DB_NAME = "HtmlPackerCache"
    const DB_VERSION = 1;
    const ASSET_STORE_NAME = "wasm_cache";
    
    function openDb() {
        return new Promise((resolve, reject) => {
            // this
            const request = indexedDB.open(DB_NAME, DB_VERSION);
    
            request.onerror = (e) => {
                console.error("IndexedDB error: ", e.target.error);
                reject("Error opening database.");
            };
    
            // fires when db version changes
            request.onupgradeneeded = (e) => {
                const db = e.target.result;
                // creates object store to hold assets
                // keypath is like a primate key
                if (!db.objectStoreNames.contains(ASSET_STORE_NAME)) {
                    db.createObjectStore(ASSET_STORE_NAME);
                }
            }
    
            // gets the actual cached asset
            request.onsuccess = (e) => {
                resolve(e.target.result);
            }
        })
    } 

    async function getAssetFromCache(db, key) {
        return new Promise((resolve, reject) => {
            // start readonly transaction to get data faster
            const transaction = db.transaction([ASSET_STORE_NAME], 'readonly');
            const store = transaction.objectStore(ASSET_STORE_NAME);
            const request = store.get(key);

            request.onerror = (e) => reject(
                "Error reading from cache: ", e.target.error);
            request.onsuccess = (e) => resolve(e.target.result);
        });
    }

    async function saveAssetToCache(db, key, value) {
        return new Promise((resolve, reject) => {
            // readwrite for saving data
            const transaction = db.transaction([ASSET_STORE_NAME], 'readwrite');
            const store = transaction.objectStore(ASSET_STORE_NAME);
            const request = store.put(value, key);

            request.onerror = (e) => reject(
                "Error writing to cache: ", e.target.error);
            request.onsuccess = (e) => {
                console.log(`Asset with key '${key}' cached.`);
                resolve();
            };
        });
    }
})();





/*
* app.js
* main app function
* orchestrates the wasm loading process with detailed status updates
*/

async function runApp() {
    // loading screen with progress tracking
    const loadingScreen = createLoadingScreen();
    console.log("Starting WASM application...");
    
    try {
        // Pass the loading screen to setupWasm for status updates
        await window.setupWasm(loadingScreen);
    } catch (error) {
        console.error("Fatal error starting WASM application:", error);
        loadingScreen.updateText("Error loading application. Please refresh the page.", 'error');
        return; // Don't hide the loading screen on error
    }
    
    // hide loading screen once WASM is loaded
    loadingScreen.updateText("Application ready!", 'success');
    setTimeout(() => {
        loadingScreen.hide();
    }, 500); // Short delay to show "ready" message
}

// prevent right click
document.addEventListener('contextmenu', event => event.preventDefault());

// run app when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);

/*
* decoder.js
* decodes the embedded application with status updates
*/

(async () => {
    // check WASM binary header for validity
    function checkMagicBytes(bytes) {
        const magic = Array.from(bytes.slice(0, 4));
        const p = magic.map(b => "0x" + b.toString(16).padStart(2, '0')).join(' ');
        console.log("WASM binary magic bytes:", p);
        
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
        base64 = base64.replace(/\s/g, '');
        const binaryString = atob(base64);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }
        return bytes;
    }
    
    async function loadDecoder(db, statusCallback) {
        console.log("Loading wasm decoder...");
        statusCallback?.("Loading decoder module...");
        
        const wasmDecoderElement = document.getElementById('bin-wasm-decoder');
        const wasmDecoderHash = wasmDecoderElement.getAttribute('hash');
        let wasmBytes;
        
        if (!wasmDecoderHash) {
            console.log("No hash found for decoder, can't access indexedDB.");
            statusCallback?.("Decoding decoder module (no cache)...");
        } else {
            console.log(wasmDecoderHash);
            const cacheKey = `wasm-decoder-${wasmDecoderHash}`;
            
            statusCallback?.("Checking cache for decoder...");
            const cachedBytes = await getAssetFromCache(db, cacheKey);

            if (cachedBytes) {
                console.log("Decoder module loaded from IndexedDB cache.");
                statusCallback?.("Decoder loaded from cache");
                wasmBytes = cachedBytes;
            } else {
                console.log("Decoder module not found in cache. Decoding...");
                statusCallback?.("Decoding decoder module...");
                const wasmDecoder64 = wasmDecoderElement.innerHTML.trim();
                wasmBytes = b64ToBytes(wasmDecoder64);
                checkMagicBytes(wasmBytes);
                
                statusCallback?.("Caching decoder module...");
                try {
                    await saveAssetToCache(db, cacheKey, wasmBytes);
                } catch(e) {
                    console.warn("Error caching decoder: ", e);
                }
            }
        }

        statusCallback?.("Initializing decoder module...");
        await wasm_decoder(wasmBytes);
        statusCallback?.("Decoder ready");
    }

    async function loadApp(db, statusCallback) {
        console.log("Loading wasm main app...");
        statusCallback?.("Loading main application...");
        
        let wasmAppElement = document.getElementById('bin-wasm-app');
        const wasmAppHash = wasmAppElement.getAttribute('hash');
        let wasmBytes;

        if (!wasmAppHash) {
            console.log("No hash found for app, can't access indexedDB.");
            statusCallback?.("Decompressing application (no cache)...");
        } else {
            console.log(wasmAppHash);
            const cacheKey = `wasm-app-${wasmAppHash}`;
            
            statusCallback?.("Checking cache for application...");
            const cachedBytes = await getAssetFromCache(db, cacheKey);

            if (cachedBytes) {
                console.log("App module loaded from IndexedDB cache.");
                statusCallback?.("Application loaded from cache");
                wasmBytes = cachedBytes;
            } else {
                console.log("App module not found in cache. Decoding...");
                statusCallback?.("Decoding application data...");
                const wasmApp64 = wasmAppElement.innerHTML.trim();
                const b = b64ToBytes(wasmApp64);
                
                statusCallback?.("Decompressing application...");
                wasmBytes = await wasm_decoder.decompress(b);
                checkMagicBytes(wasmBytes);
                
                statusCallback?.("Caching application...");
                try {
                    await saveAssetToCache(db, cacheKey, wasmBytes);
                } catch(e) {
                    console.warn("Error caching app: ", e);
                }
            }
        }

        statusCallback?.("Initializing application...");
        await wasm_bindgen(wasmBytes);
        statusCallback?.("Application initialized");
    }
    
    // IndexedDB constants
    const DB_NAME = "HtmlPackerCache";
    const DB_VERSION = 1;
    const ASSET_STORE_NAME = "wasm_cache";
    
    function openDb(statusCallback) {
        return new Promise((resolve, reject) => {
            statusCallback?.("Opening cache database...");
            const request = indexedDB.open(DB_NAME, DB_VERSION);
    
            request.onerror = (e) => {
                console.error("IndexedDB error: ", e.target.error);
                reject("Error opening database.");
            };
    
            request.onupgradeneeded = (e) => {
                const db = e.target.result;
                if (!db.objectStoreNames.contains(ASSET_STORE_NAME)) {
                    statusCallback?.("Creating cache store...");
                    db.createObjectStore(ASSET_STORE_NAME);
                }
            };
    
            request.onsuccess = (e) => {
                statusCallback?.("Cache database ready");
                resolve(e.target.result);
            };
        });
    }

    async function getAssetFromCache(db, key) {
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([ASSET_STORE_NAME], 'readonly');
            const store = transaction.objectStore(ASSET_STORE_NAME);
            const request = store.get(key);

            request.onerror = (e) => reject("Error reading from cache: ", e.target.error);
            request.onsuccess = (e) => resolve(e.target.result);
        });
    }

    async function saveAssetToCache(db, key, value) {
        return new Promise((resolve, reject) => {
            const transaction = db.transaction([ASSET_STORE_NAME], 'readwrite');
            const store = transaction.objectStore(ASSET_STORE_NAME);
            const request = store.put(value, key);

            request.onerror = (e) => reject("Error writing to cache: ", e.target.error);
            request.onsuccess = (e) => {
                console.log(`Asset with key '${key}' cached.`);
                resolve();
            };
        });
    }
    
    // Main setup function exposed globally
    window.setupWasm = async function(loadingScreen) {
        // Create a status callback function if loadingScreen is provided
        const updateStatus = loadingScreen ? 
            (text) => loadingScreen.updateText(text) : 
            (text) => console.log(`Status: ${text}`);
        
        try {
            console.log("Setting up WASM application...");
            updateStatus("Initializing...");
            
            const db = await openDb(updateStatus);
            await loadDecoder(db, updateStatus);
            await loadApp(db, updateStatus);
            
            console.log("WASM module initialized successfully!");
        } catch (e) {
            console.error("WASM setup error:", e);
            throw e;
        }
    };
})();

/*
* loading.js
* enhanced loading screen with status indicators
*/

function createLoadingScreen() {
    // Create loading screen container
    const loadingScreen = document.createElement('div');
    loadingScreen.id = 'loading-screen';
    
    // Create spinner element
    const spinner = document.createElement('div');
    spinner.className = 'spinner';
    
    // Create loading text
    const loadingText = document.createElement('div');
    loadingText.className = 'loading-text';
    loadingText.textContent = 'Loading WASM application...';
    
    // Create progress indicator
    const progressBar = document.createElement('div');
    progressBar.className = 'progress-bar';
    const progressFill = document.createElement('div');
    progressFill.className = 'progress-fill';
    progressBar.appendChild(progressFill);
    
    // Create a style element for our CSS
    const styleElement = document.createElement('style');
    styleElement.textContent = `
        #loading-screen {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background-color: #f8f9fa;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            z-index: 9999;
            transition: opacity 0.35s;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
        }
        
        .spinner {
            width: 80px;
            height: 80px;
            border: 4px solid #e9ecef;
            border-top: 4px solid #007bff;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
            margin-bottom: 24px;
        }
        
        .loading-text {
            font-size: 18px;
            font-weight: 500;
            color: #495057;
            margin-bottom: 20px;
            text-align: center;
            min-height: 28px;
            transition: color 0.3s;
        }
        
        .loading-text.error {
            color: #dc3545;
        }
        
        .loading-text.success {
            color: #28a745;
        }
        
        .progress-bar {
            width: 300px;
            height: 6px;
            background-color: #e9ecef;
            border-radius: 3px;
            overflow: hidden;
            opacity: 0.8;
        }
        
        .progress-fill {
            height: 100%;
            background-color: #007bff;
            width: 0%;
            transition: width 0.3s ease;
            border-radius: 3px;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        @media (max-width: 480px) {
            .progress-bar {
                width: 80%;
                max-width: 300px;
            }
            
            .loading-text {
                font-size: 16px;
                padding: 0 20px;
            }
        }
    `;
    
    // Append elements to the DOM
    loadingScreen.appendChild(spinner);
    loadingScreen.appendChild(loadingText);
    loadingScreen.appendChild(progressBar);
    document.head.appendChild(styleElement);
    document.body.appendChild(loadingScreen);
    
    // Track progress steps
    const progressSteps = [
        'Initializing...',
        'Opening cache database...',
        'Cache database ready',
        'Loading decoder module...',
        'Checking cache for decoder...',
        'Decoder loaded from cache',
        'Decoding decoder module...',
        'Caching decoder module...',
        'Initializing decoder module...',
        'Decoder ready',
        'Loading main application...',
        'Checking cache for application...',
        'Application loaded from cache',
        'Decoding application data...',
        'Decompressing application...',
        'Caching application...',
        'Initializing application...',
        'Application initialized',
        'Application ready!'
    ];
    
    let currentStep = 0;
    
    // Return an object with methods to control the loading screen
    return {
        // Update the loading text with progress tracking
        updateText: (text, type = 'normal') => {
            loadingText.textContent = text;
            loadingText.className = `loading-text ${type}`;
            
            // Update progress bar based on known steps
            const stepIndex = progressSteps.findIndex(step => 
                text.toLowerCase().includes(step.toLowerCase().split('...')[0])
            );
            
            if (stepIndex !== -1) {
                currentStep = Math.max(currentStep, stepIndex);
                const progress = ((currentStep + 1) / progressSteps.length) * 100;
                progressFill.style.width = `${progress}%`;
            }
        },
        
        // Hide the loading screen
        hide: () => {
            loadingScreen.style.opacity = '0';
            setTimeout(() => {
                loadingScreen.style.display = 'none';
                loadingScreen.remove();
                styleElement.remove();
            }, 500);
        },
        
        // Show the loading screen (in case it was hidden)
        show: () => {
            loadingScreen.style.display = 'flex';
            setTimeout(() => {
                loadingScreen.style.opacity = '1';
            }, 10);
        }
    };
}

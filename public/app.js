// main app function
async function runApp() {
    try {
        await setupWasm();
    } catch (error) {
        console.error("Error loading WASM module:", error);
        const errorMsg = document.createElement("div");
        errorMsg.style.color = "red";
        errorMsg.textContent = `Error calling WebAssembly function: ${error.message}`;
        wasmContainer.appendChild(errorMsg);
    }
}

// run app when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);

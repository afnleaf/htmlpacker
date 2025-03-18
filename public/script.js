

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

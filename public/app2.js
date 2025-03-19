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
            transition: opacity 0.5s;
        }
        
        .spinner {
            width: 50px;
            height: 50px;
            border: 5px solid #e9ecef;
            border-top: 5px solid #007bff;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin-bottom: 20px;
        }
        
        .loading-text {
            font-size: 18px;
            color: #495057;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    `;
    
    // Append elements to the DOM
    loadingScreen.appendChild(spinner);
    loadingScreen.appendChild(loadingText);
    document.head.appendChild(styleElement);
    document.body.appendChild(loadingScreen);
    
    // Return an object with methods to control the loading screen
    return {
        // Update the loading text
        updateText: (text) => {
            loadingText.textContent = text;
        },
        // Hide the loading screen
        hide: () => {
            loadingScreen.style.opacity = '0';
            setTimeout(() => {
                loadingScreen.style.display = 'none';
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

// main app function
async function runApp() {
    // loading screen
    const loadingScreen = createLoadingScreen();
    console.log("Starting WASM application...");
    try {
        await setupWasm();
    } catch (error) {
        // have to catch the non error of using exceptions as control flow in bevy
        console.error("Fatal error starting WASM application:", error);
        //loadingScreen.updateText("Error loading application. Please refresh the page.");
    }
    // hide loading screen once WASM is loaded
    loadingScreen.updateText("Application ready!");
    setTimeout(() => {
        loadingScreen.hide();
    }, 50); // Short delay to show "ready" message
}

// run app when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);


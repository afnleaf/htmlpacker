
// main app function
async function runApp() {
    // loading screen
    const loadingScreen = createLoadingScreen();
    console.log("Starting WASM application...");
    try {
        await window.setupWasm();
    } catch (error) {
        // have to catch the non error of using exceptions as control flow in bevy
        console.error("Fatal error starting WASM application:", error);
        //loadingScreen.updateText("Error loading application. Please refresh the page.");
    }
    // hide loading screen once WASM is loaded
    loadingScreen.updateText("Application ready!");
    setTimeout(() => {
        loadingScreen.hide();
    }, 100); // Short delay to show "ready" message
}

// prevent right click?
// should be from bindgen?
document.addEventListener('contextmenu', event => event.preventDefault());
// run app when the page is loaded
window.addEventListener('DOMContentLoaded', runApp);


// atob is the browser built in base64 decoder
function decodeBase64(base) {
    try {
        return atob(base);
    } catch (e) {
        console.log("Error decoding Base64", error);
        return null;
    }
}

// add out hello world
const a = document.createElement("div");
a.innerHTML = "dude...";
document.body.appendChild(a);

const b = document.getElementById("bin");
const c = b.innerHTML;

const d = document.createElement("div");
d.innerHTML = decodeBase64(c);
if (d) {
    document.body.appendChild(d);
}


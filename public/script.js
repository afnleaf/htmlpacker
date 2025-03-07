// atob is the browser built in base64 decoder
function decodeBase64Text(base) {
    try {
        return atob(base);
    } catch (e) {
        console.log("Error decoding Base64", error);
        return null;
    }
}

function decodeBase64PNG(base) {
    try {
        return (base);
    } catch (e) {
        console.log("Error decoding Base64", error);
        return null;
    }
}

// add out hello world
const a = document.createElement("div");
a.innerHTML = "dude...";
document.body.appendChild(a);

let b = document.getElementById("bin-text");
const c = b.innerHTML;

const d = document.createElement("div");
d.innerHTML = decodeBase64Text(c);
if (d) {
    document.body.appendChild(d);
}

// add a picture
const img = document.createElement("img");
const base64DataPNG = document.getElementById("bin-png").innerHTML.trim();
img.src = `data:image/png;base64,${base64DataPNG}`;
img.width = 368;
img.height = 547;
if (img) {
    document.body.appendChild(img);
}

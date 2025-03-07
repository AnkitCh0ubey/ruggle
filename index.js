console.log("Quering /api/search")
fetch("/api/search", {
    method:'POST',
    headers:{'Content-Type': 'text/plain'},
    body: 'glsl function for 3d noise',
}).then((response) => console.log(response))
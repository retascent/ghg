import init from "./wasm/ghg.js"

init().then(wasm => {
    window.WASM = wasm;
});

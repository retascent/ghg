import init from "../pkg/ghg.js"

init().then(wasm => {
    window.WASM = wasm;
});

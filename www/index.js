// NOTE TO DEVELOPER: If this import isn't found, chance are your build didn't output to the correct directory.
// Either add the flag `--out-dir www/wasm` to wasm-pack, or manually copy the wasm and js files to /www/wasm/
import init from './wasm/ghg.js'

// If this isn't found, chances are you didn't clone with --recurse-submodules.
// import * as png_metadata from './png-metadata/';

function remove_overlay() {
    let overlay = document.getElementById('loading_overlay');
    let fadeOutMs = parseInt(getComputedStyle(overlay).getPropertyValue('--fade-out').slice(0, -2)); // `ms` suffix

    overlay.style.opacity = '0';
    setTimeout(() => overlay.remove(), fadeOutMs);
}

init().then(wasm => {
    remove_overlay();
    window.WASM = wasm;
});

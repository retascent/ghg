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

// WIP trying to load the PNG files with metadata
// function loadFileAsBlob(url){
//     return new Promise((resolve, reject) => {
//         let xhr = new XMLHttpRequest();
//         xhr.open('GET', url, true);
//         xhr.responseType = 'blob';
//         xhr.onload = function (e) {
//             if (this.status === 200) {
//                 resolve(this.response);
//             } else {
//                 reject(this.response);
//             }
//         };
//         xhr.send();
//     })
// }
//
// const blob = await loadFileAsBlob('images/earth_temp/difference_1980_2021.blob')
// const buffer = await blob.arrayBuffer()
// console.log(blob)
// console.log(png_metadata)
// let metadata = png_metadata.readMetadata(buffer)
// console.log(metadata)

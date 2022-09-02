# Heavily under development

# Building

- Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/). More information available [here](https://github.com/rustwasm/wasm-pack).
- Run `wasm-pack` (or create a run configuration) with this command: `--target web --out-dir www/wasm`
  - `--target web`: Build for the web directly, without need for a bundler. Example [here](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html).
  - `--out-dir www/wasm`: Output build artifacts (e.g. `.wasm` and `.js` files) directly in the `www` directory, so the website can load them easily.

Then just open `www/index.html` in your favorite (supported) browser and you should be good to go!

If additional steps are required, or if anything doesn't work as expected, please open an [issue](https://github.com/retascent/ghg/issues/new/choose) or a [PR](https://github.com/retascent/ghg/compare).
You can also email me at [`retascent@gmail.com`](mailto:retascent@gmail.com).

# Heavily under development

Find the latest deployment [here](https://retascent.github.io/ghg/).

# About the Project

As a systems programmer, traditionally in C++, I find Rust very satisfying and fun.
However, since most systems programming jobs are still based in C++ instead of Rust, I want a fun and engaging project to work on for myself.

I have also meddled (both as a professional and a hobbyist) with WASM, and I have enjoyed that as well.
So here is a combination of those two passions, along with some 3D programming (which I also love), and various related topics.

The Project Temporarily Known As GHG is intended to be a climate change data visualization tool.
Interactivity and slick visuals tend to make for superb education tools, [if done correctly](https://eater.net/quaternions).

# Building GHG

- Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/). More information available [here](https://github.com/rustwasm/wasm-pack).
- Run `wasm-pack` (or create a run configuration) with this command: `--target web --out-dir www/wasm`
  - `--target web`: Build for the web directly, without need for a bundler. Example [here](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html).
  - `--out-dir www/wasm`: Output build artifacts (e.g. `.wasm` and `.js` files) directly in the `www` directory, so the website can load them easily.

Then just open `www/index.html` in your favorite (supported) browser and you should be good to go!

If additional steps are required, or if anything doesn't work as expected, please open an [issue](https://github.com/retascent/ghg/issues/new/choose) or a [PR](https://github.com/retascent/ghg/compare).
You can also email me at [`retascent@gmail.com`](mailto:retascent@gmail.com).

# Binary Projects

A few additional projects exist in `/src/bin/`. Below is some information about them:

## `texture_splitter`

A work-in-progress for manually downscaling and splitting up large images to make them more palatable to the web.
The terrain and color maps that I use are 1/16 the area of the originals, because the originals are way too big to download quickly.
I plan to use the downscaled versions for when the user is zoomed out, and to dynamically pull in high-resolution pieces of the visible area when the user zooms in.

## `merra2_inst_2d_data_export`

Another work-in-progress for the data pipeline needed for this project.
Most of the data I have gathered so far has been in HDF5 (or similar) format, and that isn't easy to just pull in and parse inside the browser.
Instead, I am build a data gathering and processing step to pull these files and export them in reasonable formats, such as images.
The images can then be easily mapped as textures to the GPU for display once they're fetched in the browser.

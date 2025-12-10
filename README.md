# htmlpacker
embed all dependencies into a single html file

a new form of offline, OS agnostic, browser based application

put the wasm in the base64 lil bro

## resources
- [utf-8](https://en.wikipedia.org/wiki/UTF-8)
- [unicode table](https://www.utf8-chartable.de/)
- [base64](https://en.wikipedia.org/wiki/Base64)
- [base94](https://vorakl.com/articles/base94/)
- [binary conversion](https://vorakl.com/articles/stream-encoding/)
- ~~[base94 - py](https://github.com/vorakl/base94)~~
- ~~[base94 - C](https://gist.github.com/iso2022jp/4054241) 👀~~
- [base122](https://github.com/kevinAlbs/Base122)


## features
- [x] text
- [x] png
- [x] custom html
- [x] css
- [x] favicon svg
- [x] wasm simple
- [x] wasm-bindgen
- [x] wasm canvas
- [x] wasm bevy
- [x] loading screen steps
- [x] indexedDB during first time load, cache wasm_modules as u8
- [ ] library api
- [ ] test suite
- [x] metadata
- [x] favicon svg
- [ ] favicon all
- [ ] lazy loading
- [x] brotli compression
- [ ] wasm advanced
- [ ] cli tool
- [ ] mcp
- [ ] ~~skip compilation~~
- [ ] ~~base94 (slowww...)~~
- [ ] base122 (in rust)
- [x] instanced mesh custom render shader pipeline
- [ ] auto webgpu turn on -> webgl2 fallback (storage buffer = bad)
- [ ] error on webgpu not found
- ???

With these current implemented features, we have a solid backbone for the htmlpacker. future iterations will depend on optimized base94 encode/decode. Right now it is too slow to be practical.

## todo
- [x] fps counter
- [x] encode textures
- [x] encode models/meshes/3d
- [x] fix default runtime / make it more intuitive to use
- [ ] simple asset loader for textures
- asset loader for models
- basis universal 
- encode fonts
- big png
- new animation
- basic game
- single wasm-binary self loading

# Implementation Plan
*this maybe doesn't need to be here?*

## Prio 1
**refactor entire code into group n+2 mode**

## Prio 2
*Optimization*

**Implement High-Performance Encoding & Decoding in Base122**

**Build a Complete Demo ("Basic Game")**

**Implement Advanced Asset Loading**
-   **Goal:** Natively support fonts and enable efficient, compressed textures as outlined in the `todo` list.
-   **Code Required:**
    -   **For Fonts (in `htmlpacker/src/htmlpacker.rs`):**
        -   When a `.ttf` or `.otf` file is specified in the YAML, read the file, Base64-encode it, and dynamically generate a CSS `@font-face` rule inside the main `<style>` tag. This is a standard and effective technique.
    -   **For Basis Universal Textures (in `wasm_modules/src/lib.rs`):**
        -   This is primarily a task for the Bevy application. You will need to add a Rust crate for Basis Universal decoding to `wasm_modules/Cargo.toml`.
        -   Modify your asset loading and material creation systems within Bevy to recognize `.basis` files, decompress them, and create `Image` assets suitable for the GPU.

## Prio 3

**engine specific optimization**

**bevy lazy loading, main engine vs assets**

**Cache Assets when loaded? IndexDB for everything embedded?**

## compile
wasm: `wasm-pack build --target no-modules`

## notes
what computer you compile on will leak bevy crate stuff, how to prevent?

```js
const wasmModule = await WebAssembly.compile(wasmBytes);
wasmBytes = wasmModule;

//Error:  DataCloneError: Failed to execute 'put' on 'IDBObjectStore': A WebAssembly.Module can not be serialized for storage.
```
- browser protects against caching compiled output
- would cause errors if wasm runtime and cached output mismatch








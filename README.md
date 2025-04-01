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
- [base94 - py](https://github.com/vorakl/base94)
- [base94 - C](https://gist.github.com/iso2022jp/4054241) 👀


## features
- [x] text
- [x] png
- [x] wasm simple
- [x] wasm-bindgen
- [x] wasm canvas
- [x] wasm bevy
- [x] loading screen simple
- [ ] loading screen advanced
- [x] metadata
- [x] favicon svg
- [ ] favicon all
- [ ] lazy loading
- [x] brotli compression
- [ ] base94 (slowww...)
- [ ] cache api assets
- [ ] wasm advanced
- ???

With these current implemented features, we have a solid backbone for the htmlpacker. future iterations will depend on optimized base94 encode/decode. Right now it is too slow to be practical.

## todo
- [x] fps counter
- [x] encode textures
- [x] encode models/meshes/3d
- [ ] simple asset loader for textures
- asset loader for models
- basis universal 
- encode fonts
- big png
- new animation
- basic game
- single wasm-binary self loading

## compile
wasm: `wasm-pack build --target no-modules`

## notes
what computer you compile on will leak bevy crate stuff, how to prevent?




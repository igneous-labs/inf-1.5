# js runtime for executing postbuild.js
NODE ?= node

.DEFAULT_GOAL := postbuild

postbuild: prod
	cd postbuild && $(NODE) postbuild.mjs

prod: clean
	wasm-pack build --target web --release --weak-refs --reference-types --out-name index -s sanctumso

clean:
	rm -rf pkg

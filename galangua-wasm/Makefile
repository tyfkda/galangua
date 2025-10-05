
.PHONY: all
all:	build

.PHONY: initialize
initialize:	install-wasm-pack
	cd www && npm install

.PHONY: install-wasm-pack
install-wasm-pack:
	cargo install wasm-pack

.PHONY: build
build:
	wasm-pack build --release

.PHONY: clean
clean:
	cargo clean

.PHONY: start-server
start-server:
	cd www && npm run start

.PHONY: deploy
deploy:
	cd www && npm run deploy

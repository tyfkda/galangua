
.PHONY: all
all:	build

.PHONY: initialize
initialize:
	cd www && npm install

.PHONY: build
build:
	wasm-pack build

.PHONY: start-server
start-server:
	cd www && npm run start

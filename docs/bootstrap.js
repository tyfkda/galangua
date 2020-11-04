/******/ (function(modules) { // webpackBootstrap
/******/ 	// install a JSONP callback for chunk loading
/******/ 	function webpackJsonpCallback(data) {
/******/ 		var chunkIds = data[0];
/******/ 		var moreModules = data[1];
/******/
/******/
/******/ 		// add "moreModules" to the modules object,
/******/ 		// then flag all "chunkIds" as loaded and fire callback
/******/ 		var moduleId, chunkId, i = 0, resolves = [];
/******/ 		for(;i < chunkIds.length; i++) {
/******/ 			chunkId = chunkIds[i];
/******/ 			if(Object.prototype.hasOwnProperty.call(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 				resolves.push(installedChunks[chunkId][0]);
/******/ 			}
/******/ 			installedChunks[chunkId] = 0;
/******/ 		}
/******/ 		for(moduleId in moreModules) {
/******/ 			if(Object.prototype.hasOwnProperty.call(moreModules, moduleId)) {
/******/ 				modules[moduleId] = moreModules[moduleId];
/******/ 			}
/******/ 		}
/******/ 		if(parentJsonpFunction) parentJsonpFunction(data);
/******/
/******/ 		while(resolves.length) {
/******/ 			resolves.shift()();
/******/ 		}
/******/
/******/ 	};
/******/
/******/
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// object to store loaded and loading chunks
/******/ 	// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 	// Promise = chunk loading, 0 = chunk loaded
/******/ 	var installedChunks = {
/******/ 		"main": 0
/******/ 	};
/******/
/******/
/******/
/******/ 	// script path function
/******/ 	function jsonpScriptSrc(chunkId) {
/******/ 		return __webpack_require__.p + "" + chunkId + ".bootstrap.js"
/******/ 	}
/******/
/******/ 	// object to store loaded and loading wasm modules
/******/ 	var installedWasmModules = {};
/******/
/******/ 	function promiseResolve() { return Promise.resolve(); }
/******/
/******/ 	var wasmImportObjects = {
/******/ 		"../pkg/galangua_wasm_bg.wasm": function() {
/******/ 			return {
/******/ 				"./galangua_wasm_bg.js": {
/******/ 					"__wbg_playse_88ad62e8ba2af1a5": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_playse_88ad62e8ba2af1a5"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_number_new": function(p0f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_number_new"](p0f64);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_CanvasRenderingContext2d_5b86ec94bce38d5b": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_CanvasRenderingContext2d_5b86ec94bce38d5b"](p0i32);
/******/ 					},
/******/ 					"__wbg_setfillStyle_2da87acf76dcbbcb": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setfillStyle_2da87acf76dcbbcb"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_drawImage_ed0ab75dd31bf26b": function(p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_drawImage_ed0ab75dd31bf26b"](p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64);
/******/ 					},
/******/ 					"__wbg_fillRect_e9ad0b5dde70ab3b": function(p0i32,p1f64,p2f64,p3f64,p4f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fillRect_e9ad0b5dde70ab3b"](p0i32,p1f64,p2f64,p3f64,p4f64);
/******/ 					},
/******/ 					"__wbg_restore_be383cadf1440d72": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_restore_be383cadf1440d72"](p0i32);
/******/ 					},
/******/ 					"__wbg_save_6d43ca6041c1ddb6": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_save_6d43ca6041c1ddb6"](p0i32);
/******/ 					},
/******/ 					"__wbg_rotate_1fae86d712dcdfd3": function(p0i32,p1f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_rotate_1fae86d712dcdfd3"](p0i32,p1f64);
/******/ 					},
/******/ 					"__wbg_translate_458add1387a34577": function(p0i32,p1f64,p2f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_translate_458add1387a34577"](p0i32,p1f64,p2f64);
/******/ 					},
/******/ 					"__wbg_getElementById_0cb6ad9511b1efc0": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getElementById_0cb6ad9511b1efc0"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_set_e0c72ee4d5eea3d5": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_e0c72ee4d5eea3d5"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlCanvasElement_4f5b5ec6cd53ccf3": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_HtmlCanvasElement_4f5b5ec6cd53ccf3"](p0i32);
/******/ 					},
/******/ 					"__wbg_width_a22f9855caa54b53": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_width_a22f9855caa54b53"](p0i32);
/******/ 					},
/******/ 					"__wbg_height_9a404a6b3c61c7ef": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_height_9a404a6b3c61c7ef"](p0i32);
/******/ 					},
/******/ 					"__wbg_getContext_37ca0870acb096d9": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getContext_37ca0870acb096d9"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_setonload_ab35a7a2495b1678": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonload_ab35a7a2495b1678"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setonerror_b91169e64312f1fa": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonerror_b91169e64312f1fa"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setsrc_4e562fe2dd3f545a": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setsrc_4e562fe2dd3f545a"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_new_6c05171898e5da27": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_6c05171898e5da27"]();
/******/ 					},
/******/ 					"__wbg_headers_d4301f4888b4640a": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_headers_d4301f4888b4640a"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithstrandinit_d1de1bfcd175e38a": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newwithstrandinit_d1de1bfcd175e38a"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Response_328c03967a8e8902": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Response_328c03967a8e8902"](p0i32);
/******/ 					},
/******/ 					"__wbg_text_966d07536ca6ccdc": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_text_966d07536ca6ccdc"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_adf3196bdc02b386": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Window_adf3196bdc02b386"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_6cc8d0b87c0a99b9": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_document_6cc8d0b87c0a99b9"](p0i32);
/******/ 					},
/******/ 					"__wbg_fetch_91f098921cc7cca8": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fetch_91f098921cc7cca8"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_7f083efc6bc6752c": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_error_7f083efc6bc6752c"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_f3b8a801d5d4b079": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newnoargs_f3b8a801d5d4b079"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_8e95613cc6524977": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_8e95613cc6524977"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_d713ea0274dfc6d2": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_d713ea0274dfc6d2"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_call_acb1ec2343d35cab": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_acb1ec2343d35cab"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_new_3e06d4f36713e4cb": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_3e06d4f36713e4cb"]();
/******/ 					},
/******/ 					"__wbg_set_304f2ec1a3ab3b79": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_304f2ec1a3ab3b79"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_resolve_2529512c3bb73938": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_resolve_2529512c3bb73938"](p0i32);
/******/ 					},
/******/ 					"__wbg_then_4a7a614abbbe6d81": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_4a7a614abbbe6d81"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_then_3b7ac098cfda2fa5": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_3b7ac098cfda2fa5"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_self_07b2f89e82ceb76d": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_self_07b2f89e82ceb76d"]();
/******/ 					},
/******/ 					"__wbg_window_ba85d88572adc0dc": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_window_ba85d88572adc0dc"]();
/******/ 					},
/******/ 					"__wbg_globalThis_b9277fc37e201fe5": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_globalThis_b9277fc37e201fe5"]();
/******/ 					},
/******/ 					"__wbg_global_e16303fe83e1d57f": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_global_e16303fe83e1d57f"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbg_self_1c83eb4471d9eb9b": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_self_1c83eb4471d9eb9b"]();
/******/ 					},
/******/ 					"__wbg_require_5b2b5b594d809d9f": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_require_5b2b5b594d809d9f"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_crypto_c12f14e810edcaa2": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_crypto_c12f14e810edcaa2"](p0i32);
/******/ 					},
/******/ 					"__wbg_msCrypto_679be765111ba775": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_msCrypto_679be765111ba775"](p0i32);
/******/ 					},
/******/ 					"__wbg_getRandomValues_05a60bf171bfc2be": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getRandomValues_05a60bf171bfc2be"](p0i32);
/******/ 					},
/******/ 					"__wbg_getRandomValues_3ac1b33c90b52596": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getRandomValues_3ac1b33c90b52596"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_randomFillSync_6f956029658662ec": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_randomFillSync_6f956029658662ec"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_static_accessor_MODULE_abf5ae284bffdf45": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_static_accessor_MODULE_abf5ae284bffdf45"]();
/******/ 					},
/******/ 					"__wbindgen_number_get": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_number_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_string_get": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_string_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper238": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_closure_wrapper238"](p0i32,p1i32,p2i32);
/******/ 					}
/******/ 				}
/******/ 			};
/******/ 		},
/******/ 	};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/ 	// This file contains only the entry chunk.
/******/ 	// The chunk loading function for additional chunks
/******/ 	__webpack_require__.e = function requireEnsure(chunkId) {
/******/ 		var promises = [];
/******/
/******/
/******/ 		// JSONP chunk loading for javascript
/******/
/******/ 		var installedChunkData = installedChunks[chunkId];
/******/ 		if(installedChunkData !== 0) { // 0 means "already installed".
/******/
/******/ 			// a Promise means "currently loading".
/******/ 			if(installedChunkData) {
/******/ 				promises.push(installedChunkData[2]);
/******/ 			} else {
/******/ 				// setup Promise in chunk cache
/******/ 				var promise = new Promise(function(resolve, reject) {
/******/ 					installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 				});
/******/ 				promises.push(installedChunkData[2] = promise);
/******/
/******/ 				// start chunk loading
/******/ 				var script = document.createElement('script');
/******/ 				var onScriptComplete;
/******/
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.src = jsonpScriptSrc(chunkId);
/******/
/******/ 				// create error before stack unwound to get useful stacktrace later
/******/ 				var error = new Error();
/******/ 				onScriptComplete = function (event) {
/******/ 					// avoid mem leaks in IE.
/******/ 					script.onerror = script.onload = null;
/******/ 					clearTimeout(timeout);
/******/ 					var chunk = installedChunks[chunkId];
/******/ 					if(chunk !== 0) {
/******/ 						if(chunk) {
/******/ 							var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 							var realSrc = event && event.target && event.target.src;
/******/ 							error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 							error.name = 'ChunkLoadError';
/******/ 							error.type = errorType;
/******/ 							error.request = realSrc;
/******/ 							chunk[1](error);
/******/ 						}
/******/ 						installedChunks[chunkId] = undefined;
/******/ 					}
/******/ 				};
/******/ 				var timeout = setTimeout(function(){
/******/ 					onScriptComplete({ type: 'timeout', target: script });
/******/ 				}, 120000);
/******/ 				script.onerror = script.onload = onScriptComplete;
/******/ 				document.head.appendChild(script);
/******/ 			}
/******/ 		}
/******/
/******/ 		// Fetch + compile chunk loading for webassembly
/******/
/******/ 		var wasmModules = {"0":["../pkg/galangua_wasm_bg.wasm"]}[chunkId] || [];
/******/
/******/ 		wasmModules.forEach(function(wasmModuleId) {
/******/ 			var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/
/******/ 			// a Promise means "currently loading" or "already loaded".
/******/ 			if(installedWasmModuleData)
/******/ 				promises.push(installedWasmModuleData);
/******/ 			else {
/******/ 				var importObject = wasmImportObjects[wasmModuleId]();
/******/ 				var req = fetch(__webpack_require__.p + "" + {"../pkg/galangua_wasm_bg.wasm":"579ba1e8bfe6c2d0be9b"}[wasmModuleId] + ".module.wasm");
/******/ 				var promise;
/******/ 				if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 					promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 						return WebAssembly.instantiate(items[0], items[1]);
/******/ 					});
/******/ 				} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 					promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 				} else {
/******/ 					var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 					promise = bytesPromise.then(function(bytes) {
/******/ 						return WebAssembly.instantiate(bytes, importObject);
/******/ 					});
/******/ 				}
/******/ 				promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 					return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 				}));
/******/ 			}
/******/ 		});
/******/ 		return Promise.all(promises);
/******/ 	};
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// on error function for async loading
/******/ 	__webpack_require__.oe = function(err) { console.error(err); throw err; };
/******/
/******/ 	// object with all WebAssembly.instance exports
/******/ 	__webpack_require__.w = {};
/******/
/******/ 	var jsonpArray = window["webpackJsonp"] = window["webpackJsonp"] || [];
/******/ 	var oldJsonpFunction = jsonpArray.push.bind(jsonpArray);
/******/ 	jsonpArray.push = webpackJsonpCallback;
/******/ 	jsonpArray = jsonpArray.slice();
/******/ 	for(var i = 0; i < jsonpArray.length; i++) webpackJsonpCallback(jsonpArray[i]);
/******/ 	var parentJsonpFunction = oldJsonpFunction;
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./bootstrap.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./bootstrap.js":
/*!**********************!*\
  !*** ./bootstrap.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("// A dependency graph that contains any wasm must all be imported\n// asynchronously. This `bootstrap.js` file does the single async import, so\n// that no one else needs to worry about it again.\n__webpack_require__.e(/*! import() */ 0).then(__webpack_require__.bind(null, /*! ./index.js */ \"./index.js\"))\n  .catch(e => console.error(\"Error importing `index.js`:\", e));\n\n\n//# sourceURL=webpack:///./bootstrap.js?");

/***/ })

/******/ });
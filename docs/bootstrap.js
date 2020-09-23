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
/******/ 					"__wbindgen_cb_forget": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_cb_forget"](p0i32);
/******/ 					},
/******/ 					"__wbg_playse_ac7385354855f7d5": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_playse_ac7385354855f7d5"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_number_new": function(p0f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_number_new"](p0f64);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_CanvasRenderingContext2d_619282746f6101d3": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_CanvasRenderingContext2d_619282746f6101d3"](p0i32);
/******/ 					},
/******/ 					"__wbg_setfillStyle_52278f0fc6e7e85f": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setfillStyle_52278f0fc6e7e85f"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_drawImage_28e5b8eca6c34bc3": function(p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_drawImage_28e5b8eca6c34bc3"](p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64);
/******/ 					},
/******/ 					"__wbg_fillRect_8c22b56874211a38": function(p0i32,p1f64,p2f64,p3f64,p4f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fillRect_8c22b56874211a38"](p0i32,p1f64,p2f64,p3f64,p4f64);
/******/ 					},
/******/ 					"__wbg_restore_a595d0d680c7393c": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_restore_a595d0d680c7393c"](p0i32);
/******/ 					},
/******/ 					"__wbg_save_ffbc979fe70b5415": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_save_ffbc979fe70b5415"](p0i32);
/******/ 					},
/******/ 					"__wbg_rotate_82b98a46bc5e44d9": function(p0i32,p1f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_rotate_82b98a46bc5e44d9"](p0i32,p1f64);
/******/ 					},
/******/ 					"__wbg_translate_7353a876d037e772": function(p0i32,p1f64,p2f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_translate_7353a876d037e772"](p0i32,p1f64,p2f64);
/******/ 					},
/******/ 					"__wbg_getElementById_66a113a03886aac6": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getElementById_66a113a03886aac6"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_set_38cd18b5c3b1bd2d": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_38cd18b5c3b1bd2d"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlCanvasElement_0feb941e3d100079": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_HtmlCanvasElement_0feb941e3d100079"](p0i32);
/******/ 					},
/******/ 					"__wbg_width_8ac7ccf7f1856cdb": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_width_8ac7ccf7f1856cdb"](p0i32);
/******/ 					},
/******/ 					"__wbg_height_58d21d690b204c43": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_height_58d21d690b204c43"](p0i32);
/******/ 					},
/******/ 					"__wbg_getContext_a8aab8274f84fca2": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getContext_a8aab8274f84fca2"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_setonload_f35002b2488461d3": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonload_f35002b2488461d3"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setonerror_eb00bf5798315cb9": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonerror_eb00bf5798315cb9"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setsrc_7e888e4ced74b27b": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setsrc_7e888e4ced74b27b"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_new_0599e1276155199a": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_0599e1276155199a"]();
/******/ 					},
/******/ 					"__wbg_headers_53617ff036220d47": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_headers_53617ff036220d47"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithstrandinit_f6f97e155b57ae94": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newwithstrandinit_f6f97e155b57ae94"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Response_7d08290905bb6381": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Response_7d08290905bb6381"](p0i32);
/******/ 					},
/******/ 					"__wbg_text_bb2ab1ec910d6485": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_text_bb2ab1ec910d6485"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_747b56d25bab9510": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Window_747b56d25bab9510"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_c9bb82e72b87972b": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_document_c9bb82e72b87972b"](p0i32);
/******/ 					},
/******/ 					"__wbg_fetch_adfca4043cbf7a28": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fetch_adfca4043cbf7a28"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_44d97cfce214d7c7": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_error_44d97cfce214d7c7"](p0i32);
/******/ 					},
/******/ 					"__wbg_log_d85e484a8ba03c98": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_log_d85e484a8ba03c98"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_714dec97cfe3da72": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newnoargs_714dec97cfe3da72"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_652fa4cfce310118": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_652fa4cfce310118"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_0d50cec2d58307ad": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_0d50cec2d58307ad"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_call_56e03f05ec7df758": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_56e03f05ec7df758"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_new_2a149ff291bf4137": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_2a149ff291bf4137"]();
/******/ 					},
/******/ 					"__wbg_set_c7b5e4d8ec7a9ff4": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_c7b5e4d8ec7a9ff4"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_resolve_607ba012325a12c4": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_resolve_607ba012325a12c4"](p0i32);
/******/ 					},
/******/ 					"__wbg_then_a44670e94672e44d": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_a44670e94672e44d"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_then_201b9d5deaad5d11": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_201b9d5deaad5d11"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_self_8a533577b0c752d3": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_self_8a533577b0c752d3"]();
/******/ 					},
/******/ 					"__wbg_window_5912543aff64b459": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_window_5912543aff64b459"]();
/******/ 					},
/******/ 					"__wbg_globalThis_8f997d48cb67f28e": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_globalThis_8f997d48cb67f28e"]();
/******/ 					},
/******/ 					"__wbg_global_69b29294e4daedff": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_global_69b29294e4daedff"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbg_self_1b7a39e3a92c949c": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_self_1b7a39e3a92c949c"]();
/******/ 					},
/******/ 					"__wbg_require_604837428532a733": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_require_604837428532a733"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_crypto_968f1772287e2df0": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_crypto_968f1772287e2df0"](p0i32);
/******/ 					},
/******/ 					"__wbg_getRandomValues_a3d34b4fee3c2869": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getRandomValues_a3d34b4fee3c2869"](p0i32);
/******/ 					},
/******/ 					"__wbg_getRandomValues_f5e14ab7ac8e995d": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getRandomValues_f5e14ab7ac8e995d"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_randomFillSync_d5bd2d655fdf256a": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_randomFillSync_d5bd2d655fdf256a"](p0i32,p1i32,p2i32);
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
/******/ 					"__wbindgen_closure_wrapper249": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_closure_wrapper249"](p0i32,p1i32,p2i32);
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
/******/ 				var req = fetch(__webpack_require__.p + "" + {"../pkg/galangua_wasm_bg.wasm":"8e9ac665fad1cc87805e"}[wasmModuleId] + ".module.wasm");
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
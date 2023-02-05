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
/******/ 					"__wbindgen_string_get": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_string_get"](p0i32,p1i32);
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
/******/ 					"__wbindgen_number_get": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_number_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_CanvasRenderingContext2d_9037c3eea625e27b": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_CanvasRenderingContext2d_9037c3eea625e27b"](p0i32);
/******/ 					},
/******/ 					"__wbg_setfillStyle_a0bd3a7496c1c5ae": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setfillStyle_a0bd3a7496c1c5ae"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_drawImage_0e5d448b4c02ba9b": function(p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_drawImage_0e5d448b4c02ba9b"](p0i32,p1i32,p2f64,p3f64,p4f64,p5f64,p6f64,p7f64,p8f64,p9f64);
/******/ 					},
/******/ 					"__wbg_fillRect_37d4341db168ab0f": function(p0i32,p1f64,p2f64,p3f64,p4f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fillRect_37d4341db168ab0f"](p0i32,p1f64,p2f64,p3f64,p4f64);
/******/ 					},
/******/ 					"__wbg_restore_2eda799771bbdaf3": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_restore_2eda799771bbdaf3"](p0i32);
/******/ 					},
/******/ 					"__wbg_save_88e5b8eebd3f0de5": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_save_88e5b8eebd3f0de5"](p0i32);
/******/ 					},
/******/ 					"__wbg_rotate_a756fbbe1a13cb2e": function(p0i32,p1f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_rotate_a756fbbe1a13cb2e"](p0i32,p1f64);
/******/ 					},
/******/ 					"__wbg_translate_3b6341171a005432": function(p0i32,p1f64,p2f64) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_translate_3b6341171a005432"](p0i32,p1f64,p2f64);
/******/ 					},
/******/ 					"__wbg_getElementById_0c9415d96f5b9ec6": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getElementById_0c9415d96f5b9ec6"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_set_23d56ff06768e13b": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_23d56ff06768e13b"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlCanvasElement_7b561bd94e483f1d": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_HtmlCanvasElement_7b561bd94e483f1d"](p0i32);
/******/ 					},
/******/ 					"__wbg_width_ad2acb326fc35bdb": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_width_ad2acb326fc35bdb"](p0i32);
/******/ 					},
/******/ 					"__wbg_height_65ee0c47b0a97297": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_height_65ee0c47b0a97297"](p0i32);
/******/ 					},
/******/ 					"__wbg_getContext_b506f48cb166bf26": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getContext_b506f48cb166bf26"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_setonload_8fda3afa75bfeb0d": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonload_8fda3afa75bfeb0d"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setonerror_1a08d1953fb8ad4c": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setonerror_1a08d1953fb8ad4c"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setsrc_9bc5e1e5a71b191f": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_setsrc_9bc5e1e5a71b191f"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_new_7b1587cf2acba6fc": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_7b1587cf2acba6fc"]();
/******/ 					},
/******/ 					"__wbg_headers_3618f72dcec019b7": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_headers_3618f72dcec019b7"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithstrandinit_41c86e821f771b24": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newwithstrandinit_41c86e821f771b24"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Response_e928c54c1025470c": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Response_e928c54c1025470c"](p0i32);
/******/ 					},
/******/ 					"__wbg_text_5cb78830c1a11c5b": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_text_5cb78830c1a11c5b"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_a2a08d3918d7d4d0": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_instanceof_Window_a2a08d3918d7d4d0"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_14a383364c173445": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_document_14a383364c173445"](p0i32);
/******/ 					},
/******/ 					"__wbg_fetch_23507368eed8d838": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_fetch_23507368eed8d838"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_e2677af4c7f31a14": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_error_e2677af4c7f31a14"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_is_object": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_is_object"](p0i32);
/******/ 					},
/******/ 					"__wbg_randomFillSync_64cc7d048f228ca8": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_randomFillSync_64cc7d048f228ca8"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_getRandomValues_98117e9a7e993920": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_getRandomValues_98117e9a7e993920"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_process_2f24d6544ea7b200": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_process_2f24d6544ea7b200"](p0i32);
/******/ 					},
/******/ 					"__wbg_versions_6164651e75405d4a": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_versions_6164651e75405d4a"](p0i32);
/******/ 					},
/******/ 					"__wbg_node_4b517d861cbcb3bc": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_node_4b517d861cbcb3bc"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_is_string": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_is_string"](p0i32);
/******/ 					},
/******/ 					"__wbg_modulerequire_3440a4bcf44437db": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_modulerequire_3440a4bcf44437db"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_crypto_98fc271021c7d2ad": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_crypto_98fc271021c7d2ad"](p0i32);
/******/ 					},
/******/ 					"__wbg_msCrypto_a2cdb043d2bfe57f": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_msCrypto_a2cdb043d2bfe57f"](p0i32);
/******/ 					},
/******/ 					"__wbg_self_ba1ddafe9ea7a3a2": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_self_ba1ddafe9ea7a3a2"]();
/******/ 					},
/******/ 					"__wbg_window_be3cc430364fd32c": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_window_be3cc430364fd32c"]();
/******/ 					},
/******/ 					"__wbg_globalThis_56d9c9f814daeeee": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_globalThis_56d9c9f814daeeee"]();
/******/ 					},
/******/ 					"__wbg_global_8c35aeee4ac77f2b": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_global_8c35aeee4ac77f2b"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_fc5356289219b93b": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newnoargs_fc5356289219b93b"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_306ce8d57919e6ae": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_306ce8d57919e6ae"]();
/******/ 					},
/******/ 					"__wbg_call_4573f605ca4b5f10": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_4573f605ca4b5f10"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_9855a4612eb496cb": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_9855a4612eb496cb"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_call_8e1338b908441bd2": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_call_8e1338b908441bd2"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_set_b12cd0ab82903c2f": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_b12cd0ab82903c2f"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_buffer_de1150f91b23aa89": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_buffer_de1150f91b23aa89"](p0i32);
/******/ 					},
/******/ 					"__wbg_resolve_f269ce174f88b294": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_resolve_f269ce174f88b294"](p0i32);
/******/ 					},
/******/ 					"__wbg_then_1c698eedca15eed6": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_1c698eedca15eed6"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_then_4debc41d4fc92ce5": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_then_4debc41d4fc92ce5"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_new_97cf52648830a70d": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_new_97cf52648830a70d"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithlength_e833b89f9db02732": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_newwithlength_e833b89f9db02732"](p0i32);
/******/ 					},
/******/ 					"__wbg_subarray_9482ae5cd5cd99d3": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_subarray_9482ae5cd5cd99d3"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_length_e09c0b925ab8de5d": function(p0i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_length_e09c0b925ab8de5d"](p0i32);
/******/ 					},
/******/ 					"__wbg_set_a0172b213e2469e9": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbg_set_a0172b213e2469e9"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_memory": function() {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_memory"]();
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper283": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/galangua_wasm_bg.js"].exports["__wbindgen_closure_wrapper283"](p0i32,p1i32,p2i32);
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
/******/ 				var req = fetch(__webpack_require__.p + "" + {"../pkg/galangua_wasm_bg.wasm":"9aa22ee59687c94c566c"}[wasmModuleId] + ".module.wasm");
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
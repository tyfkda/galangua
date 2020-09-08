import { WasmAppFramework, WasmRenderer } from 'galangua-wasm'

const renderer = WasmRenderer.new('mycanvas')
const framework = WasmAppFramework.new(
  renderer,
  function get_now() {
    return Date.now()
  },
  function get_item(key) {
    return localStorage.getItem(key)
  },
  function set_item(key, value) {
    localStorage.setItem(key, value)
  })

document.addEventListener('keydown', (event) => {
  framework.on_key(event.code, true)
})
document.addEventListener('keyup', (event) => {
  framework.on_key(event.code, false)
})

const loop = () => {
  framework.update()
  framework.draw()
  requestAnimationFrame(loop)
}
requestAnimationFrame(loop)

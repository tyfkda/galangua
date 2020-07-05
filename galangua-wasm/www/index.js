import { WasmAppFramework, WasmRenderer } from 'galangua-wasm'

const renderer = WasmRenderer.new('mycanvas')
const framework = WasmAppFramework.new(renderer)

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

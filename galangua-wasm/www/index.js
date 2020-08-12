import { WasmAppFramework, WasmRenderer } from 'galangua-wasm'

const CANVAS_ID = 'mycanvas'

function fitCanvas() {
  const canvas = document.getElementById(CANVAS_ID)
  if (canvas.width / canvas.height >= window.innerWidth / window.innerHeight) {
    canvas.style.width = `100%`
    canvas.style.height = `${canvas.height * window.innerWidth / canvas.width}px`
  } else {
    canvas.style.height = `100%`
    canvas.style.width = `${canvas.width * window.innerHeight / canvas.height}px`
  }
}

function disableBounce() {
  document.addEventListener('touchmove', (event) => event.preventDefault(), {passive: false})
}

function setupTouchButtons() {
  const setTouchHandler = function(id, callback) {
    const elem = document.getElementById(id)
    elem.addEventListener('touchstart', (e) => { /*e.preventDefault();*/ e.stopPropagation(); callback(true); return false; })
    elem.addEventListener('touchend', (_) => callback(false))
    elem.addEventListener('touchleave', (_) => callback(false))
  }

  const holder = document.getElementById('touch-btn-holder')

  try {
    document.createEvent("TouchEvent")

    // Touch enable
    setTouchHandler('left-btn', (down) => framework.on_touch(-1, down))
    setTouchHandler('right-btn', (down) => framework.on_touch(1, down))
    setTouchHandler('shot-btn', (down) => framework.on_touch(100, down))
    holder.style.display = ''
  } catch (_e) {
    // Touch disable
    holder.style.display = 'none'
  }
}

function setupResizeListener() {
  window.addEventListener('resize', (_) => {
    fitCanvas()
  })
}

fitCanvas()
disableBounce()
setupTouchButtons()
setupResizeListener()

const renderer = WasmRenderer.new(CANVAS_ID)
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



document.documentElement.addEventListener('touchend', (event) => {
  event.preventDefault();
}, false);

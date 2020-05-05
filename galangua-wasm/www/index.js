import {WasmAppFramework, WasmRenderer} from 'galangua-wasm'
import {audioManager} from './audio_manager'

const CHANNEL_COUNT = 3

const AUDIO_ASSETS = [
  'assets/audio/se_get_1',
  'assets/audio/se_pyuun',
  'assets/audio/se_zugyan',
  'assets/audio/jingle_1up',
]
const ENALBE_AUDIO = 'assets/audio/se_get_1'

const CANVAS_ID = 'mycanvas'

window.play_se = function play_se(channel, filename) {
  audioManager.playSe(channel, filename)
}

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
    elem.addEventListener('touchstart', (e) => { /*e.preventDefault();*/ e.stopPropagation(); callback(true); return false; }, {passive: true})
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

function createCoverScreen(title) {
  const cover = document.createElement('div')
  cover.className = 'centering'
  cover.style.position = 'absolute'
  cover.style.left = cover.style.top = cover.style.right = cover.style.bottom = '0'
  cover.style.backgroundColor = 'rgba(0,0,0,0.5)'
  cover.style.color = 'white'
  cover.style.textAlign = 'center'
  cover.innerText = title

  document.body.appendChild(cover)
  return cover
}

fitCanvas()
disableBounce()
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

function loop() {
  framework.update()
  framework.draw()
  requestAnimationFrame(loop)
}

const cover = createCoverScreen('Loading...')
audioManager.createContext(CHANNEL_COUNT)
audioManager.loadAllAudios(AUDIO_ASSETS)
  .then(() => {
    cover.innerText = 'Galangua\n\nTouch to start'      

    const onClick = () => {
      audioManager.playSe(0, ENALBE_AUDIO)
      cover.removeEventListener('click', onClick)
      cover.removeEventListener('touchstart', onClick)
      document.body.removeChild(cover)

      setupTouchButtons()

      requestAnimationFrame(loop)
    }
    cover.addEventListener('click', onClick)
    cover.addEventListener('touchstart', onClick, {passive: true})
  })

document.documentElement.addEventListener('touchend', (event) => {
  event.preventDefault();
}, false);

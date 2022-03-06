import { READY, CANVAS } from './types'

function start() {
  const canvas = document.querySelector('canvas')
  const offscreen = canvas.transferControlToOffscreen()

  const worker = new Worker(new URL('./worker', import.meta.url))
  worker.onmessage = ({ data: { type } }) => {
    switch (type) {
      case READY:
        worker.postMessage({ type: CANVAS, payload: { canvas: offscreen } }, [offscreen])
        break
    }
  }
}

start()
import { Layer } from 'ol/layer'
import { READY, CANVAS } from './types'

export class OffscreenLayer extends Layer {
  createContainer() {
    const container = document.createElement('div')
    container.style.position = 'absolute'
    container.style.width = '100%'
    container.style.height = '100%'
    const canvas = document.createElement('canvas')
    canvas.style.display = 'block'
    canvas.style.width = '100%'
    canvas.style.height = '100%'
    container.appendChild(canvas)

    const worker = new Worker(new URL('./worker', import.meta.url))
    worker.onmessage = ({ data: { type } }) => {
      switch (type) {
        case READY:
          const offscreen = canvas.transferControlToOffscreen()
          offscreen.width = canvas.clientWidth
          offscreen.height = canvas.clientHeight
          worker.postMessage({ type: CANVAS, payload: {
            canvas: offscreen
          } }, [offscreen])
          break
      }
    }

    return container
  }

  createRenderer() {
    return {
      ready: true
    }
  }

  render() {
    if (!this.container_) {
      this.container_ = this.createContainer()
    }
    return this.container_
  }
}

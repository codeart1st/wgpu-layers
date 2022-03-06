import * as wasm from 'wgpu-layers'

import { READY, CANVAS } from './types'

let canvas

// https://github.com/gfx-rs/wgpu/issues/1986
self.Window = WorkerGlobalScope
self.window = self
self.window.document = {
  querySelectorAll: () => {
    return [canvas]
  }
}

self.onmessage = ({ data: { type, payload } }) => {
  switch (type) {
    case CANVAS:
      canvas = payload.canvas
      wasm.start(canvas)
      break
  }
}

self.postMessage({ type: READY })

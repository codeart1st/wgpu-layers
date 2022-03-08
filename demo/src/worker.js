import init, { initThreadPool, start } from 'wgpu-layers'

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

self.onmessage = async ({ data: { type, payload } }) => {
  switch (type) {
    case CANVAS:
      canvas = payload.canvas
      await start(canvas)
      break
  }
}

async function run() {
  await init()
  await initThreadPool(navigator.hardwareConcurrency)

  self.postMessage({ type: READY })
}

run()

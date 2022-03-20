import init, { initThreadPool, start } from 'wgpu-layers'

import { READY, CANVAS, FRAME_STATE } from './types'

let canvas, render

// https://github.com/gfx-rs/wgpu/issues/1986
self.Window = WorkerGlobalScope

self.onmessage = async ({ data: { type, payload } }) => {
  switch (type) {
    case CANVAS:
      canvas = payload.canvas
      render = await start(canvas)
      break
    case FRAME_STATE:
      const { coordinateToPixelTransform } = payload
      render && render(
        getViewMatrix(coordinateToPixelTransform, canvas.width, canvas.height)
      )
      break
  }
}

function getViewMatrix(coordinateToPixelTransform, width, height) {
  const half_width = width * .5
  const half_height = height * .5
  return [
    coordinateToPixelTransform[0] / half_width, -(coordinateToPixelTransform[1] / half_height), 0.0, 0.0,
    coordinateToPixelTransform[2] / half_width, -(coordinateToPixelTransform[3] / half_height), 0.0, 0.0,
    coordinateToPixelTransform[4] / half_width - 1.0, -(coordinateToPixelTransform[5] / half_height - 1.0), 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]
}

async function run() {
  await init()
  await initThreadPool(navigator.hardwareConcurrency - 1)

  self.postMessage({ type: READY })
}

run()

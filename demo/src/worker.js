import init, { initThreadPool, start, render, addPbfTileData } from 'wgpu-layers'
import { create, makeInverse } from 'ol/transform'

import { READY, CANVAS, FRAME_STATE, RENDERED, PBF_DATA } from './types'

let canvas, ready = false

// https://github.com/gfx-rs/wgpu/issues/1986
self.Window = WorkerGlobalScope

self.onmessage = async ({ data: { type, payload } }) => {
  switch (type) {
    case CANVAS:
      canvas = payload.canvas
      await start(canvas, new Uint8Array(payload.data))
      ready = true
      break
    case FRAME_STATE:
      const { frameState: { size, viewState } } = payload
      requestAnimationFrame(() => {
        if (ready) {
          const [width, height] = size
          if (canvas.width != width || canvas.height != height) {
            canvas.width = width
            canvas.height = height
          }
          render(
            getViewMatrix(viewState, canvas.width, canvas.height),
            size
          )
        }
        self.postMessage({
          type: RENDERED, payload: {
            frameState: {
              viewState
            }
          }
        })
      })
      break
    case PBF_DATA:
      const { data, tileCoord, extent } = payload
      if (ready) {
        addPbfTileData(new Uint8Array(data), tileCoord, extent)
      }
      break
  }
}

function getViewMatrix(viewState, width, height) {
  const halfWidth = width * .5
  const halfHeight = height * .5

  const scaleX = viewState.resolution * halfWidth
  const scaleY = viewState.resolution * halfHeight

  const viewTransform = create()

  const sin = Math.sin(viewState.rotation)
  const cos = Math.cos(viewState.rotation)

  viewTransform[0] = scaleX * cos
  viewTransform[1] = scaleX * sin
  viewTransform[2] = scaleY * -sin
  viewTransform[3] = scaleY * cos
  viewTransform[4] = viewState.center[0]
  viewTransform[5] = viewState.center[1]

  const viewMatrix = makeInverse(
    create(),
    viewTransform
  )

  return [
    viewMatrix[0], viewMatrix[1], 0.0, 0.0,
    viewMatrix[2], viewMatrix[3], 0.0, 0.0,
    viewMatrix[4], viewMatrix[5], 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]
}

async function run() {
  await init()
  await initThreadPool(navigator.hardwareConcurrency - 1)

  self.postMessage({ type: READY })
}

run()

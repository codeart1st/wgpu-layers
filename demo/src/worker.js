import init, { initThreadPool, start } from 'wgpu-layers'
import { create, multiply, makeInverse, compose } from 'ol/transform'

import { READY, CANVAS, FRAME_STATE, RENDERED } from './types'

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
      const { frameState: { size, viewState, coordinateToPixelTransform } } = payload
      requestAnimationFrame(() => {
        if (render) {
          const [width, height] = size
          if (canvas.width != width || canvas.height != height) {
            canvas.width = width
            canvas.height = height
          }
          render(
            getViewMatrix(coordinateToPixelTransform, canvas.width, canvas.height),
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
  /*const worldTransform = compose(
    create(),
    half_width,
    half_height,
    half_width,
    -half_height,
    0,
    0,
    0
  )
  const inverseWorldTransform = makeInverse(
    create(),
    worldTransform
  )
  const transform = multiply(
    inverseWorldTransform,
    coordinateToPixelTransform
  )

  return [
    transform[0], transform[1], 0.0, 0.0,
    transform[2], transform[3], 0.0, 0.0,
    transform[4], transform[5], 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]*/
}

async function run() {
  await init()
  await initThreadPool(navigator.hardwareConcurrency - 1)

  self.postMessage({ type: READY })
}

run()

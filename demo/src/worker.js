import init, { initThreadPool, startWithOffscreenCanvas, render, addPbfTileData } from 'wgpu-layers'
import { create, makeInverse } from 'ol/transform'

import { READY, STARTED, CANVAS, SHARED_ARRAY_BUFFER, PBF_DATA } from './types'

let shared_state, ready = false

self.onmessage = async ({ data: { type, payload } }) => {
  switch (type) {
    case CANVAS:
      await startWithOffscreenCanvas(payload.canvas)
      self.postMessage({ type: STARTED })
      ready = true
      loop()
      break
    case SHARED_ARRAY_BUFFER:
      shared_state = payload
      break
    case PBF_DATA:
      const { data, tileCoord, extent } = payload
      if (ready) {
        await addPbfTileData(new Uint8Array(data), tileCoord, extent)
      }
      break
  }
}

function loop() {
  Atomics.wait(new Int32Array(shared_state), 6, 0) // wait until notify

  const { size, viewState } = getFrameState()

  const [width, height] = size
  render(
    getViewMatrix(viewState, width, height),
    size
  )

  setTimeout(loop)
}

function getFrameState() {
  const slice = new Uint32Array(shared_state)

  const buffer = new ArrayBuffer(shared_state.byteLength)
  const f32_buffer = new Float32Array(buffer)
  const uint32_buffer = new Uint32Array(buffer)

  for (let i = 0; i < uint32_buffer.length; i++) {
    uint32_buffer[i] = Atomics.load(slice, i)
  }

  return {
    size: [uint32_buffer[0], uint32_buffer[1]],
    viewState: {
      center: [f32_buffer[2], f32_buffer[3]],
      resolution: f32_buffer[4],
      rotation: f32_buffer[5]
    }
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
    0.0, 0.0, 1.0, 0.0,
    viewMatrix[4], viewMatrix[5], 0.0, 1.0
  ]
}

async function run() {
  await init()
  await initThreadPool(navigator.hardwareConcurrency - 1)

  self.postMessage({ type: READY })
}

run()

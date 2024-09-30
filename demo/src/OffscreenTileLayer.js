import { Tile } from 'ol/layer'
import { getUid } from 'ol/util'
import TileState from 'ol/TileState'
import { READY, STARTED, CANVAS, SHARED_ARRAY_BUFFER, PBF_DATA } from './types'

export class OffscreenTileLayer extends Tile {
  constructor(opt_options) {
    super(opt_options)
    this.queue = []
    this.shared_state = new SharedArrayBuffer(7 * Uint32Array.BYTES_PER_ELEMENT)
  }

  createContainer() {
    const container = document.createElement('div')
    container.style.position = 'absolute'
    container.style.width = '100%'
    container.style.height = '100%'
    const offscreenCanvas = document.createElement('canvas')
    offscreenCanvas.style.display = 'block'
    offscreenCanvas.style.position = 'absolute'
    offscreenCanvas.style.width = '100%'
    offscreenCanvas.style.height = '100%'
    container.appendChild(offscreenCanvas)

    this.offscreenCanvas = offscreenCanvas

    this.worker = new Worker(new URL('./worker', import.meta.url), {
      type: 'module'
    })

    this.worker.postMessage({
      type: SHARED_ARRAY_BUFFER,
      payload: this.shared_state
    })

    this.worker.onmessage = async ({ data: { type } }) => {
      switch (type) {
        case READY:
          const offscreen = offscreenCanvas.transferControlToOffscreen()

          this.worker.postMessage({
            type: CANVAS, payload: {
              canvas: offscreen
            }
          }, [offscreen])
          break
        case STARTED:
          this.queue.forEach(task => task())
          this.queue = []
          this.ready = true
          break
      }
    }

    return container
  }

  createRenderer() {
    return {
      ready: true,
      prepareFrame: this.prepareFrame.bind(this),
      renderFrame: this.render.bind(this)
    }
  }

  async pushPbfTileData(pbf, tileCoord, extent) {
    const message = {
      type: PBF_DATA,
      payload: {
        data: pbf,
        tileCoord,
        extent
      }
    }
    if (this.ready) {
      this.worker.postMessage(message, [pbf])
      return Promise.resolve()
    }
    return new Promise((resolve, reject) => {
      try {
        this.queue.push(() => {
          this.worker.postMessage(message, [pbf])
          resolve()
        })
      } catch (e) {
        reject(e)
      }
    })
  }

  loadTiles(frameState) {
    const viewState = frameState.viewState
    const projection = viewState.projection
    const pixelRatio = frameState.pixelRatio
    const viewResolution = viewState.resolution
    const tileQueue = frameState.tileQueue

    const tileSource = this.getSource()
    const tileGrid = tileSource.getTileGridForProjection(projection)
    const z = tileGrid.getZForResolution(viewResolution, tileSource.zDirection)

    const tileRange = tileGrid.getTileRangeForExtentAndZ(frameState.extent, z)

    const tileSourceKey = getUid(tileSource)
    if (!(tileSourceKey in frameState.wantedTiles)) {
      frameState.wantedTiles[tileSourceKey] = {}
    }
    const wantedTiles = frameState.wantedTiles[tileSourceKey]

    const tileResolution = tileGrid.getResolution(z)

    for (let x = tileRange.minX; x <= tileRange.maxX; ++x) {
      for (let y = tileRange.minY; y <= tileRange.maxY; ++y) {
        const tile = tileSource.getTile(z, x, y, pixelRatio, projection)
        if (tile.getState() == TileState.IDLE) {
          wantedTiles[tile.getKey()] = true
          if (!tileQueue.isKeyQueued(tile.getKey())) {
            tileQueue.enqueue([
              tile,
              tileSourceKey,
              tileGrid.getTileCoordCenter(tile.tileCoord),
              tileResolution,
            ])
          }
        }
      }
    }
  }

  setFrameState({ size, viewState: { center, resolution, rotation } }) {
    const slice = new Uint32Array(this.shared_state)

    const buffer = new ArrayBuffer(this.shared_state.byteLength)
    const f32_buffer = new Float32Array(buffer)
    const uint32_buffer = new Uint32Array(buffer)

    uint32_buffer[0] = size[0]
    uint32_buffer[1] = size[1]

    f32_buffer[2] = center[0]
    f32_buffer[3] = center[1]
    f32_buffer[4] = resolution
    f32_buffer[5] = rotation

    for (let i = 0; i < uint32_buffer.length; i++) {
      Atomics.store(slice, i, uint32_buffer[i])
    }

    Atomics.notify(new Int32Array(this.shared_state), 6, 1)
  }

  renderDeferred(frameState) {
    // empty
  }

  prepareFrame(frameState) {
    return this.ready
  }

  render(frameState) {
    if (!this.container_) {
      this.container_ = this.createContainer()
    }

    this.loadTiles(frameState)
    this.setFrameState(frameState)

    frameState.animate = !this.ready // TODO: track pbf data fully loaded
    return this.container_
  }
}

import { Tile } from 'ol/layer'
import { compose, create, toString as toTransformString } from 'ol/transform'
import { getUid } from 'ol/util'
import TileState from 'ol/TileState'
import { READY, CANVAS, FRAME_STATE, RENDERED } from './types'

export class OffscreenTileLayer extends Tile {
  // Transform the container to account for the differnece between the (newer)
  // main thread frameState and the (older) worker frameState
  updateContainerTransform() {
    if (this.workerFrameState) {
      const viewState = this.mainThreadFrameState.viewState
      const renderedViewState = this.workerFrameState.viewState
      const center = viewState.center
      const resolution = viewState.resolution
      const rotation = viewState.rotation
      const renderedCenter = renderedViewState.center
      const renderedResolution = renderedViewState.resolution
      const renderedRotation = renderedViewState.rotation
      const transform = create()
      compose(
        transform,
        (renderedCenter[0] - center[0]) / resolution,
        (center[1] - renderedCenter[1]) / resolution,
        renderedResolution / resolution,
        renderedResolution / resolution,
        rotation - renderedRotation,
        0,
        0
      )
      this.transformContainer.style.transform = toTransformString(transform)
    }
  }

  async getTestVectorTileArrayBuffer() {
    const url = 'https://tegola-osm-demo.go-spatial.org/v1/maps/osm/1/1/1'
    const response = await fetch(url)
    return await response.arrayBuffer()
  }

  createContainer() {
    const container = document.createElement('div')
    container.style.position = 'absolute'
    container.style.width = '100%'
    container.style.height = '100%'
    this.transformContainer = document.createElement('div')
    this.transformContainer.style.position = 'absolute'
    this.transformContainer.style.width = '100%'
    this.transformContainer.style.height = '100%'
    const offscreenCanvas = document.createElement('canvas')
    offscreenCanvas.style.display = 'block'
    offscreenCanvas.style.position = 'absolute'
    offscreenCanvas.style.visibility = 'hidden'
    offscreenCanvas.style.width = '100%'
    offscreenCanvas.style.height = '100%'
    const canvas = document.createElement('canvas')
    canvas.style.display = 'block'
    canvas.style.position = 'absolute'
    canvas.style.width = '100%'
    canvas.style.height = '100%'
    container.appendChild(this.transformContainer)
    container.appendChild(offscreenCanvas)
    this.transformContainer.appendChild(canvas)

    const ctx = canvas.getContext('2d')

    this.canvas = canvas
    this.offscreenCanvas = offscreenCanvas
    this.ctx = ctx

    this.worker = new Worker(new URL('./worker', import.meta.url))
    this.worker.onmessage = async ({ data: { type, payload } }) => {
      switch (type) {
        case READY:
          const offscreen = offscreenCanvas.transferControlToOffscreen()

          offscreen.width = offscreenCanvas.clientWidth
          offscreen.height = offscreenCanvas.clientHeight

          canvas.width = offscreenCanvas.clientWidth
          canvas.height = offscreenCanvas.clientHeight

          const vectorTile = await this.getTestVectorTileArrayBuffer()

          this.worker.postMessage({
            type: CANVAS, payload: {
              canvas: offscreen,
              data: vectorTile
            }
          }, [offscreen, vectorTile])
          break
        case RENDERED:
          requestAnimationFrame(() => {
            ctx.clearRect(0, 0, offscreenCanvas.width, offscreenCanvas.height)
            ctx.drawImage(offscreenCanvas, 0, 0)

            this.workerFrameState = payload.frameState
            this.updateContainerTransform()
            this.rendering = false
          })
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

  resize(frameState) {
    const [width, height] = frameState.size
    if (this.canvas.width != width || this.canvas.height != height) {
      this.canvas.width = width
      this.canvas.height = height
      this.ctx.drawImage(this.offscreenCanvas, 0, 0)
    }
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

    const tileSourceKey = getUid(tileSource);
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

  render(frameState) {
    if (!this.container_) {
      this.container_ = this.createContainer()
    }

    this.mainThreadFrameState = frameState
    this.updateContainerTransform()

    this.resize(frameState)
    this.loadTiles(frameState)

    if (!this.rendering) {
      this.rendering = true
      this.worker.postMessage({
        type: FRAME_STATE,
        payload: {
          frameState: {
            size: frameState.size,
            viewState: {
              center: frameState.viewState.center,
              resolution: frameState.viewState.resolution,
              rotation: frameState.viewState.rotation
            }
          }
        }
      })
    } else {
      frameState.animate = true
    }
    return this.container_
  }
}

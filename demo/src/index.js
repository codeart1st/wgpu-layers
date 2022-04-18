import { Map, View } from 'ol'
import { Tile } from 'ol/layer'
import { OSM, TileDebug, VectorTile } from 'ol/source'

import { OffscreenTileLayer } from './OffscreenTileLayer'
import { PBF_DATA } from './types'

function start() {
  const offscreenTileLayer = new OffscreenTileLayer({
    source: new VectorTile({
      tileLoadFunction: async (tile, src) => {
        const { tileCoord, extent } = tile
        const response = await fetch(src)
        const pbf = await response.arrayBuffer()
        offscreenTileLayer.worker.postMessage({
          type: PBF_DATA, payload: {
            data: pbf,
            tileCoord,
            extent
          }
        }, [pbf])
        tile.setFeatures([]) // finish tile loading
      },
      url: 'https://tegola-osm-demo.go-spatial.org/v1/maps/osm/{z}/{x}/{y}'
    })
  })
  new Map({
    target: 'map',
    layers: [
      new Tile({
        source: new OSM()
      }),
      offscreenTileLayer,
      new Tile({
        source: new TileDebug()
      })
    ],
    view: new View({
      center: [0, 0],
      zoom: 10
    })
  })
}

start()
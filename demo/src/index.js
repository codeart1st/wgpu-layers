import { Map, View } from 'ol'
import { Tile } from 'ol/layer'
import { OSM, TileDebug, VectorTile } from 'ol/source'

import { OffscreenTileLayer } from './OffscreenTileLayer'

function start() {
  const offscreenTileLayer = new OffscreenTileLayer()
  const vectorTileSource = new VectorTile({
    tileLoadFunction: async (tile, src) => {
      const { tileCoord, extent } = tile
      const response = await fetch(src)
      const pbf = await response.arrayBuffer()
      await offscreenTileLayer.pushPbfTileData(pbf, tileCoord, extent)
      tile.setFeatures([]) // finish tile loading
    },
    url: 'https://tegola-osm-demo.go-spatial.org/v1/maps/osm/{z}/{x}/{y}'
  })
  offscreenTileLayer.setSource(vectorTileSource)
  new Map({
    target: 'map',
    layers: [
      new Tile({
        source: new OSM()
      }),
      offscreenTileLayer,
      new Tile({
        source: new TileDebug({
          tileGrid: vectorTileSource.tileGrid
        })
      })
    ],
    view: new View({
      center: [1489199.332673, 6894017.412561],
      zoom: 4
    })
  })
}

start()
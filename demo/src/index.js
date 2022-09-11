import { Map, View } from 'ol'
import { Tile } from 'ol/layer'
import { createXYZ } from 'ol/tilegrid'
import { OSM, TileDebug, VectorTile } from 'ol/source'

import { OffscreenTileLayer } from './OffscreenTileLayer'

function start() {
  const offscreenTileLayer = new OffscreenTileLayer()
  const tileGrid = createXYZ({ maxZoom: 22 })
  const vectorTileSource = new VectorTile({
    tileGrid,
    zDirection: 0, // same as TileDebug
    tileLoadFunction: async (tile, src) => {
      const { tileCoord, extent } = tile
      const response = await fetch(src)
      const pbf = await response.arrayBuffer()
      await offscreenTileLayer.pushPbfTileData(pbf, tileCoord, extent)
      tile.setFeatures([]) // finish tile loading
    },
    url: 'https://tegola-osm-demo.go-spatial.org/v1/maps/osm/{z}/{x}/{y}'
  })
  vectorTileSource.getTileGridForProjection = () => { // override for better debug purposes
    return tileGrid
  }
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
          tileGrid
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
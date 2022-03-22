import { Map, View } from 'ol'
import { Tile } from 'ol/layer'
import { OSM } from 'ol/source'

import { OffscreenLayer } from './offscreen_layer'

function start() {
  const map = new Map({
    target: 'map',
    layers: [
      new Tile({
        source: new OSM()
      }),
      new OffscreenLayer({})
    ],
    view: new View({
      center: [0, 0],
      zoom: 4
    })
  })
}

start()
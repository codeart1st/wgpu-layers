import { Map, View } from 'ol'
import { Tile } from 'ol/layer'
import { OSM } from 'ol/source'
import { fromLonLat } from 'ol/proj'

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
      center: [0, 0],//[1458675.916789971, 6911404.021700942],
      zoom: 4
    })
  })
}

start()
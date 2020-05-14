<template>
    <div id='app' class='h-full w-full'>
        <div id='map' class='h-full w-full'></div>
    </div>
</template>

<script>
import mapboxgl from 'mapbox-gl';
import 'mapbox-gl/dist/mapbox-gl.css';

export default {
    name: 'RAI',
    data: function() {
        return {
            map: false
        };
    },
    mounted: function() {
        this.$nextTick(() => {
            this.init();
        });
    },
    methods: {
        init: function() {
            fetch(`${window.location.origin}/tiles`, {
                method: 'GET'
            }).then((res) => {
                if (res.status !== 200 && res.message) {
                    throw new Error(res.message);
                } else if (res.status !== 200) {
                    throw new Error('Failed to fetch map data');
                }

                return res.json();
            }).then((res) => {
                mapboxgl.accessToken = res.token;

                this.map = new mapboxgl.Map({
                    container: 'map',
                    zoom: 1,
                    bounds: res.bounds,
                    style: 'mapbox://styles/mapbox/light-v9'
                });

                this.map.on('load', () => {
                    this.map.addSource('rai-coverage', {
                        type: 'vector',
                        tiles: [
                            `${window.location.origin}/tiles/coverage/{z}/{x}/{y}`
                        ],
                        minzoom: 0,
                        maxzoom: 16
                    });

                    this.map.addSource('rai-network', {
                        type: 'vector',
                        tiles: [
                            `${window.location.origin}/tiles/network/{z}/{x}/{y}`
                        ],
                        minzoom: 0,
                        maxzoom: 16
                    });

                    this.map.addLayer({
                        id: 'rai-coverage',
                        type: 'fill',
                        source: 'rai-coverage',
                        'source-layer': 'data',
                        layout: {},
                        paint: {
                            'fill-color': '#ff0000',
                            'fill-opacity': 0.8
                        }
                    });

                    this.map.addLayer({
                        id: 'rai-network',
                        type: 'line',
                        source: 'rai-network',
                        'source-layer': 'data',
                        'layout': {
                            'line-join': 'round',
                            'line-cap': 'round'
                        },
                        'paint': {
                            'line-color': '#888',
                            'line-width': 1
                        }
                    });
                });
            });
        }
    }
}
</script>

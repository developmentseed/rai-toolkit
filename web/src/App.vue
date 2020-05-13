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
                    style: 'mapbox://styles/mapbox/light-v9'
                });
            });
        }
    }
}
</script>

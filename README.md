<h1 align=center>RAI Toolkit</h1>

The RAI toolkit is designed to take multiple street networks, perform a rough
conflation into a single network and then perform the [RAI calculation](https://datacatalog.worldbank.org/dataset/rural-access-index-rai)

## Installation

We recommend building and using the docker image for running the toolkit.

From the terminal, run the following:

```sh
docker build -t rai .
```

Alternatively, if you have a local postgres/postgis installation and wish to run the tool
without the overhead of the docker system, you can compile the binaries manually, or
download a prebuild binary from the [release](https://github.com/developmentseed/rai-toolkit/releases) page.

## Data Pre-Req

The RAI toolkit is a self contained RAI calculator, and can perform an RAI calculation without any custom data.
By default it will use the [NASA SEDAC Product](https://sedac.ciesin.columbia.edu/) as well as street network data
from [OpenStreetMap](https://openstreetmap.org/). That said, the toolkit supports data conflation and any custom or
proprietary dataset can be used, so long as it is provided in the desired format.

### Population

A raster population dataset is required to be loaded before any RAI calculations are performed. This project
has scripts to format and load the [NASA SEDAC Product](https://sedac.ciesin.columbia.edu/)

Download the global `.asc` files into a local directory. Then, if using docker, use `docker cp` to copy the files into the running docker
container.

Once the files are avaiable, run

```
./util/cache-nasa <path-to-nasa-data>
```

This script will create the necessary RAI database structure as well as format and load the SEDAC data. Note that this data is global
so this initial import can take some time. This import is only necessary to do once. The toolkit will create mutable subsets of the data
from the master import.

### OpenStreetMap

To obtain the OpenStreetMap road network for a given country, run:

```
./util/cache-osm <ISO 3166-1 Alpha-2 Code>
```

Note: [ISO 3166-1 Alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) codes lookup table

This script will download and filter OSM data into a subset of all-weather roads. Primary/Secondary Highways are assumed to be paved.
Lower classifications of road (Residential/Unclassified) must have an explicit `surface=<paved,concrete,etc>` to be included as
all weather roads. OSM data is constantly being improved and for our reviewed countries has a high degree of accuracy.

### Data Format

If you are not using OSM data, or are conflating an additional dataset into OSM data, your data must be modified
to be Line-Delimited GeoJSON.

Modern versions of the GDAL `ogr2ogr` tool support bi-directional conversion from a wide variety of geospatial formats into
line delimited GeoJSON.

The following example would convert a shapefile into GeoJSON.

```
ogr2ogr -t_srs 'EPSG:4326' -f 'GeoJSON' country.geojsonld input.shp
```

Note: To be valid GeoJSON, the projection MUST be `EPSG:4326`.

The GeoJSON must then have the following properties:

| Property  | Description |
| --------- | ----------- |
| `name`    | Name of the road. Multiple names can be delimited via `;` |
| `highway` | Optional: If present, will filter input as OSM data |
| `surface` | Type of surface. Surfaces listed [here](https://wiki.openstreetmap.org/wiki/Key:surface) are supported.









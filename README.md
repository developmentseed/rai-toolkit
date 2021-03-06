<h1 align=center>RAI Toolkit</h1>

The RAI toolkit is designed to take multiple street networks, perform a rough
conflation into a single network and then perform the [RAI calculation](https://datacatalog.worldbank.org/dataset/rural-access-index-rai)

## User Manual
Detailed instructions on installation and [usage is available here](https://github.com/developmentseed/rai-toolkit/wiki/User-Manual).

## Data prerequisite

The RAI toolkit is a self contained RAI calculator, and can perform an RAI calculation without any custom data.
By default it will use the [NASA SEDAC Product](https://sedac.ciesin.columbia.edu/) as well as street network data
from [OpenStreetMap](https://openstreetmap.org/). That said, the toolkit supports data conflation and any custom or
proprietary dataset can be used, so long as it is provided in the desired format.

### Population

A raster population dataset is required to be loaded before any RAI calculations are performed. This project has scripts to format and load the NASA SEDAC Product. To download the global population grid:

- Visit the [data download page](https://sedac.ciesin.columbia.edu/data/set/gpw-v4-population-count-rev11/data-download)
- Select the appropriate grid size
- Download the zip file following the link
- Unzip the archive

Then, if using docker, use `docker cp` to copy the files into the running docker container by running the following command your terminal:

```
docker cp path-to-population-data-folder/ rai:/
```

Once the files are available, run:

```
./util/cache-nasa <path-to-nasa-data-folder>
```

This script will create the necessary RAI database structure as well as format and load the SEDAC data. Note that this data is global so this initial import can take some time. This import is only necessary to do once. The toolkit will create mutable subsets of the data from the master import.

### OpenStreetMap
To work with a new country, it needs to be first configured by adding to `./utils/cache-osm.conf`. Open this file in a text editor and add the [ISO 3166-1 Alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) and a link to [Geofabrik downloads](https://download.geofabrik.de/).

Then, to download the OpenStreetMap road network for a given country, run:

```
./util/cache-osm <ISO 3166-1 Alpha-2 Code>
```

This script will download and filter OSM data into a subset of all-weather roads. Primary/Secondary Highways are assumed to be paved.
Lower classifications of road (Residential/Unclassified) must have an explicit `surface=<paved,concrete,etc>` to be included as
all weather roads. OSM data is constantly being improved and for our reviewed countries has a high degree of accuracy.

### Custom road network shapefiles

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

## Toolkit Modes

The toolkit has several modules, these modules can always be listed via

```sh
rai-toolkit --help
```

The following will be returned:

```
calc        Calculate RAI
conflate    Conflate two street networks together
drop        Drop a loaded country from the database
filter      Filter OSM data to only include linestrings/highways
help        Prints this message or the help of the given subcommand(s)
list        List countries that are currently loaded
viz         Once a country is calc, open a webserver to visualize the output
```

Note: further help can always be obtained about a given subcommand by using the `--help` flag on a subcommand

*Example*

```sh
rai-toolkit conflate --help
```

### Conflate

Accept two street networks and conflate them together based on street name and geographic proximity. The output
of this mode is a single conflated line-delimited geojson file which can subsequently be used by the `calc` module.

*Example*

```sh
rai-toolkit conflate py.geojsonld py_new.geojsonld --iso py --langs es --output output.geojson
```

### Filter

The filter mode accepts a line-delimited GeoJSON representation of an OSM PBF file. The GeoJSON will initially
contain all of the OSM features in a given geographic area. The filter mode will take this file and extract all
road segments that are explicitly, or have a high probability of being all-season roads.

Generally this mode will not be used directly, but instead will be called automatically from the cache-osm utility.

*Example*

```sh
rai-toolkit filter raw_osm.geojsonld > filtered.geojsonld
```

### Calc

This module performs the RAI calculation itself based on a given all weather road network.

The module will output a covered & uncovered population metric based on how much of the population is within
2km of an all-season road.

*Example*

```sh
rai-toolkit calc \
    py.geojsonld \
    --iso py \
    --bounds py_admin.geojsonld \
    --output results.csv
```

Note: A population dataset, such as NASA SEDAC, must be loaded before the `calc` module can be used. See Data Pre-Req if this has not been done.

### Viz

The viz module will enable a simple Mapbox Vector Tile server, and a basic browser based UI. This UI shows a basic overview of
the road network and buffering calculations that were used to generate the RAI metric.

This command can only be used on a country that has already been loaded via the `calc` module.

*Example*

```sh
rai-toolkit viz --iso py
```

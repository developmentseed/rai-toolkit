<h1 align=center>RAI Toolkit</h1>

The RAI toolkit is designed to take multiple street networks, perform a rough
conflation into a single network and then perform the [RAI calculation](https://datacatalog.worldbank.org/dataset/rural-access-index-rai)

## Installation

We recommend using the docker images provided for running the toolkit. Prebuild images
are provided with each [release](https://github.com/developmentseed/rai-toolkit/releases).

Once the docker image has been downloaded, it can be loaded locally via the terminal using:

```sh
cd Downloads

docker load < rai-<version>.docker
```

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

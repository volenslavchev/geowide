# geowide

A federated system for global image-based localization. It consists of two components - an indexer, which indexes a particular h3o cell of the earth (using an image retrieval model), and a master server.
Indexers coordinate with the master server. When a user makes a localization request to the master server, it narrows down the search to a cluster of cells. It then calls all indexers inside those cells, to
find possible matching locations.

## indexer

First, a request to the OSM API is made, to get all public roads inside of the [h3o cell](https://github.com/HydroniumLabs/h3o). Then, they're divided into points, which are ideally 25 meters apart. Each point represents
a panorama image that needs to be indexed.

The indexer collects street-view panoramas from different sources (Mapillary, Google Street View), then produces their embeddings using the [MegaLoc](https://github.com/gmberton/megaloc) image retrieval model.
It finally saves the embeddings into [qdrant](https://qdrant.tech/) (a vector database). Once a panorama image is saved into the database, it's never fetched again, or erased from the database.
So you could index in separate sessions, if you need to - you'll never index the same panorama image twice.

```
Usage: indexer <COMMAND>

Commands:
  visualize  Visualize h3o cells, indexed points, and missing points
  index      Index a specific h3o cell
  serve      Serve the indexer HTTP server
  locate     Find the location of an image
  cleanup    List info about the indexer, clean up incomplete points
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## master server

TBD.
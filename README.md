# ing-csv-importer

This small utility can be used to process a CSV file as given by the ING (DiBa) account overview into an SQLite database. That database can be used to further analyze your personal expenses.

I wrote this tool because exporter CSV files as returned from the German ING website are kind of weird and need some kind of post-processing (they are encoded as `latin1`, for example). I cannot guarantee this works on any CSV file that is presented to users (perhaps international variants of ING export things differently).

Why SQLite? This tool can be executed many times, but the SQLite database will not include duplicate entries (e.g. because exported files overlap in transaction history). That could probably be done when exporting to a "complete" CSV file as well, but letting SQLite take care of this was, to be honest, much easier.

## License

This repository is [MIT licensed](./LICENSE).

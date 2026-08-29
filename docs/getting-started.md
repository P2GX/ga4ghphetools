# Getting Started

ga4ghphetools is a Rust library that is intended to support curation and analysis of GA4GH phenopackets. It can be used to support Graphical User Interface (GUI) applications such as [phenoboard](https://github.com/P2GX/phenoboard) and [phenoblend](https://github.com/P2GX/phenoblendtk), and also offers a command-line application with some functionalities.

## Usage as library

Add the crate to your project:

```toml
[dependencies]
ga4ghphetools = { git = 'https://github.com/P2GX/ga4ghphetools.git', tag = '0.4.136' }
```

Note that while ga4ghphetools is still in early development, it will be available from github; we plan to release the tool on crate.io later.

## Command line application

See [app](app.md).

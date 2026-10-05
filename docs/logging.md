# Logging

ga4ghphetools uses the Rust logging crate to provide logging capabilities. It uses the [log](https://docs.rs/log/latest/log/) crate, which is a lightweight logging facade that provides a single logging API that abstracts over the actual logging implementation. Users of ga4ghphetools can choose the logging implementation that is most suitable for the use case.

The [application](app.md) that is provided with the library provides an example using the [log4rs](https://docs.rs/log4rs/latest/log4rs/) logging implementation.

In brief, add the following to your `Cargo.toml` file

```toml
log4rs = { version = "1.4", default-features = false, features = [
    "config_parsing",
    "toml_format", 
    "console_appender", 
    "rolling_file_appender", 
    "compound_policy", 
    "size_trigger", 
    "fixed_window_roller", 
    "pattern_encoder"] }
```

(Possibly adapting the features according to the needs of your application).

Then add this line to your main file to initialize the logging framework

```rust
log4rs::init_file("log4rs.*", Default::default()).expect("Failed to initialize log4rs");
```

Note that `log4rs.*` can be a json, yaml, or toml file.

Then provide the configuration file (the `log4rs.toml` we use is shown below).

<script setup>
import log4rsConfig from './../log4rs.toml?raw'
</script>

```toml
{{ log4rsConfig }}
```


## Important

The `log4rs` framework will not create a directory, so if you want to have it write a a file called `log/my-app.log`, then you will need to create the directory `log` before running, otherwise it will silently fail.
To activate the logging function of the application provided with this library, it is necessary to create the `log` directory first (or change the path in the `log4rs.toml` file).
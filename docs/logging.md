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


```toml
refresh_rate = "30 seconds"

[appenders.stdout]
kind = "console"

[appenders.stdout.encoder]
kind = "pattern"
pattern = "{d(%Y-%m-%d %H:%M:%S)} [{l}] - {m}{n}"

[appenders.rolling_file]
kind = "rolling_file"
path = "log/phetools.log"

[appenders.rolling_file.encoder]
kind = "pattern"
pattern = "{d} [{l}] [{t}] - {m}{n}"

[appenders.rolling_file.policy]
kind = "compound"

[appenders.rolling_file.policy.trigger]
kind = "size"
limit = "20 mb"



[appenders.rolling_file.policy.roller]
kind = "delete"

[root]
level = "trace"
appenders = ["stdout", "rolling_file"]
```


## Important

The `log4rs` framework will not create a directory, so if you want to have it write a a file called `log/my-app.log`, then you will need to create the directory `log` before running, otherwise it will silently fail.
To activate the logging function of the application provided with this library, it is necessary to create the `log` directory first (or change the path in the `log4rs.toml` file).

Note also that for the command-line app, we use `kind = "delete"`,  meaning that when the log hits 20 MB, the log file is simply deleted and logging starts fresh, with no archive kept. This is fine for demonstration purposes, but may not be optimal for other applications. The the [log4rs documenation](https://docs.rs/log4rs/latest/log4rs/) for more information.

## Example

If all works well, you should see lines like the following in the log file

```text
2026-10-06T07:24:28.588291+02:00 [TRACE] [ga4ghphetools::hpo::hpo_util] - Syncing HPO label 'Intellectual disability, moderate' -> 'Moderate intellectual disability' for HP:0002342
```

The logging to the command line is provided in a simpler format.
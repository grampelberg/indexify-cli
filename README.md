# indexify-cli

Interact with the indexify API server.

## API Client

Here are a couple things you can do with the client. To see more complete
examples, take a look at [cli](src/cli). Each file is named after the resource
those commands interact with.

```rust
let client = Client::new("http://localhost:8900")

let all_namespaces = client.list::<Vec<DataNamespace>>().await?;
let one_namespace = client.get::<DataNamespace>("default").await?;

let stream = client.with_namespace("default").get_stream::<Download>("my_content_id").await?;
client.with_namespace("default").upload(ContentUpload::new("foo.txt").with_graph_names(&["one"]));
```

## CLI Utilities

- Recursive sub-commands - Allow arbitrary commands to be parents and have
  children via. clap's subcommand derive pattern. Uses `Derive(Command)` from
  [derive](derive/src/lib.rs) to add pre_run, run, next and post_run hooks for
  commands.
- Output - The `--output` flag provides [Format](src/output.rs) via global flags
  to all commands. This has a pretty mode that outputs tables and a json mode
  that does raw json output.
- File Reading - A clap value parser that can deserialize files into the
  specific type. This has json and yaml deserialization currently supported.
  Take a look at [graph.rs](src/cli/graph.rs) for an example.
- API Client - The api client is also a value parser that constructs a global
  available to every subcommand. Note that because value parsers can't know
  about each other, the namespace is not automatically added to this. Adding the
  namespace is left up to the individual commands depending on whether they need
  it or not (as not all resources are namespaced). Take a look at
  [client.rs](src/client.rs) or [root.rs](src/cli/root.rs) for how this is being
  handled.
- Progress Bar - An example of indicatif's ProgressBar being populated by
  AsyncRead transparently. See [content.rs](src/cli/content.rs).
- Telemetry - Automatic activity and error reporting.

### Telemetry

A tracing layer which can report activity and errors. It allows implementations
of the sink to be plugged out. As of now, Posthog has been implemented.

To get errors and activity out of a function, you can instrument it:

```rust
#[instrument(err, fields(activity = "name::of::activity"))]
fn foo() -> Result<(), std::error:Error> {
  Ok(())
}
```

The `err` parameter tells tracing to emit an error event if the function returns
an error. The `activity` field tells telemetry to report activity on span
creation.

Activity reporting works for events in addition to spans. You can send activity
in a function body with:

```rust
fn foo() {
  info!(activity = "name::of::activity", "my message");
}
```

The layer comes with its own filters, you'll not want to use per-registry
filtering as it'll ensure your telemetry is up to the global filtering and
levels. Luckily, `tracing_subscriber` allows for per-layer filters. To make sure
this works, you can add layers directly to the registry:

```rust
let format_layer = tracing_subscriber::fmt::layer()
  .with_env_filter(EnvFilter::from_default_env());

let telemetry_layer = telemetry::Telemetry::new(posthog::Posthog::new("key"))
  .with_activity()
  .with_errors();

tracing_subscriber::registry()
  .with(format_layer)
  .with(telemetry_layer)
  .init();
```

It is recommended that you use `build.rs` to pull in the API keys used for
handlers. As an example of how to set this up with Posthog, when you create the
layer:

```rust
static PH_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

fn main() {
  let ph = Posthog::new(PH_KEY.unwrap_or("unimplemented"));
}
```

Then, in `build.rs` you can add to the build process:

```rust
static PH_VAR: &str = "POSTHOG_API_KEY";

fn main() {
    if let Some(key) = env::var_os(PH_VAR) {
        println!("cargo::rustc-env={}={}", PH_VAR, key.to_string_lossy());
    }
}
```

#### Handler

Take a look at the `telemetry::Handler` trait. It has two functions which
construct events (`on_span` and `on_event`) and a function to publish the event.
Because tracing requires that layer handlers are synchronous, the `capture`
function is called via. tokio's `spawn_blocking`. This guarantees that any
telemetry completes before the program exits and does not block normal program
flow.

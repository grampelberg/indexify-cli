use std::env;

use eyre::Result;

static PH_VAR: &str = "POSTHOG_API_KEY";

fn main() -> Result<()> {
    if let Some(key) = env::var_os(PH_VAR) {
        println!("cargo::rustc-env={}={}", PH_VAR, key.to_string_lossy());
    }

    Ok(())
}

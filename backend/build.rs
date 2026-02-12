use std::path::Path;

fn main() {
    let env_dev = Path::new(".env.dev");
    let env_file = Path::new(".env");

    // Copy .env.dev to .env so dotenvy loads the dev config
    if env_dev.exists() {
        std::fs::copy(env_dev, env_file).expect("Failed to copy .env.dev to .env");
    }

    // Re-run build script if .env.dev changes
    println!("cargo:rerun-if-changed=.env.dev");
}

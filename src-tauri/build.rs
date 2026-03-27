use std::env;

fn main() {
    maybe_override_tauri_config_for_local_builds();
    tauri_build::build();
}

fn maybe_override_tauri_config_for_local_builds() {
    let profile = env::var("PROFILE").unwrap_or_default();
    let skip_resources = env::var("TAURI_SKIP_RESOURCES").is_ok() || profile == "test";

    if !skip_resources {
        return;
    }

    let mut merge_config = serde_json::json!({});
    if skip_resources {
        merge_config["bundle"]["resources"] = serde_json::json!([]);
    }

    match serde_json::to_string(&merge_config) {
        Ok(json) => {
            env::set_var("TAURI_CONFIG", json);
            if skip_resources {
                println!("cargo:warning=TAURI resources disabled for local build");
            }
        }
        Err(err) => {
            println!("cargo:warning=Failed to serialize TAURI_CONFIG override: {err}");
        }
    }
}

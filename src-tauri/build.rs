// Garde : 1.3.1 et 1.5.0 sont sorties sans le modele IA (application inutilisable).
// On refuse de compiler si model.onnx est absent, n'est qu'un pointeur Git LFS
// (clone sans `git lfs pull`), ou si une config passee a `tauri build --config`
// remplace bundle.resources sans le reprendre (les tableaux JSON ne fusionnent pas).
const MODEL: &str = "resources/model.onnx";
const MIN_MODEL_BYTES: u64 = 100 * 1024 * 1024;

fn main() {
    println!("cargo:rerun-if-changed={MODEL}");
    println!("cargo:rerun-if-env-changed=TAURI_CONFIG");

    let size = std::fs::metadata(MODEL).map(|m| m.len()).unwrap_or(0);
    if size < MIN_MODEL_BYTES {
        panic!(
            "{MODEL} fait {size} octets (attendu > 100 Mo) : modele absent ou pointeur Git LFS. \
             Lancez `git lfs pull`."
        );
    }

    // ponytail: test de chaine, pas de parsing JSON ; suffit tant que le modele
    // est declare via "resources/*" ou par son nom.
    if let Ok(cfg) = std::env::var("TAURI_CONFIG") {
        if cfg.contains("\"resources\"") && !cfg.contains("resources/*") && !cfg.contains("model.onnx") {
            panic!(
                "La config passee a `tauri build --config` remplace bundle.resources sans \
                 model.onnx : ajoutez \"resources/*\": \"./\" a ses resources."
            );
        }
    }

    tauri_build::build()
}

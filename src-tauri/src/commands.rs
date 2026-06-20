/// commands.rs — Commandes Tauri exposées au frontend.

use crate::{
    image_processor::{
        apply_mask, encode_base64_png, encode_png, load_image, load_image_from_bytes,
        original_preview_data_url, BackgroundColor,
    },
    ml_engine,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager};

// ─── Stockage de l'image clipboard originale (pour retraitement fond) ─────────

static CLIPBOARD_ORIGINAL: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();

fn clipboard_store() -> &'static Mutex<Option<Vec<u8>>> {
    CLIPBOARD_ORIGINAL.get_or_init(|| Mutex::new(None))
}

// ─── Types partagés ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessOptions {
    pub background: BackgroundColor,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchProgress {
    pub index: usize,
    pub total: usize,
    pub name: String,
    pub result_data_url: Option<String>,
    pub error: Option<String>,
}

/// Résultat commun original + résultat pour le split-preview.
#[derive(Debug, Serialize)]
pub struct ProcessResult {
    pub original_data_url: String,
    pub result_data_url: String,
}

// ─── Helper : init modèle ─────────────────────────────────────────────────────

fn ensure_model(app: &AppHandle) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Répertoire resources introuvable : {e}"))?;

    let model_path = resource_dir.join("model.onnx");

    ml_engine::init_model(&model_path).map_err(|e| e.to_string())
}

// ─── Commandes ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn process_single_image(
    app: AppHandle,
    path: String,
    options: ProcessOptions,
) -> Result<ProcessResult, String> {
    ensure_model(&app)?;

    let file_path = PathBuf::from(&path);
    if !file_path.exists() {
        return Err(format!("Fichier introuvable : {path}"));
    }

    let original_data_url = original_preview_data_url(&file_path).map_err(|e| e.to_string())?;

    let img = load_image(&file_path).map_err(|e| e.to_string())?;
    let mask = ml_engine::run_inference(&img).map_err(|e| e.to_string())?;
    let result_data_url = apply_mask(&img, &mask, &options.background)
        .and_then(|r| encode_base64_png(&r))
        .map_err(|e| e.to_string())?;

    Ok(ProcessResult { original_data_url, result_data_url })
}

#[tauri::command]
pub async fn process_batch_images(
    app: AppHandle,
    paths: Vec<String>,
    options: ProcessOptions,
) -> Result<(), String> {
    ensure_model(&app)?;

    let total = paths.len();
    for (index, path_str) in paths.iter().enumerate() {
        let file_path = PathBuf::from(path_str);
        let name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("inconnu")
            .to_string();

        let progress = match process_one_file(&file_path, &options) {
            Ok(data_url) => BatchProgress {
                index,
                total,
                name,
                result_data_url: Some(data_url),
                error: None,
            },
            Err(e) => BatchProgress {
                index,
                total,
                name,
                result_data_url: None,
                error: Some(e.to_string()),
            },
        };

        let _ = app.emit("batch-progress", &progress);
    }

    Ok(())
}

fn process_one_file(path: &Path, options: &ProcessOptions) -> anyhow::Result<String> {
    let img = load_image(path)?;
    let mask = ml_engine::run_inference(&img)?;
    let result = apply_mask(&img, &mask, &options.background)?;
    encode_base64_png(&result).map_err(Into::into)
}

/// Lit l'image depuis le presse-papier, la traite, et retourne
/// original + résultat pour afficher le split-preview correct.
#[tauri::command]
pub async fn process_clipboard_image(
    app: AppHandle,
    options: ProcessOptions,
) -> Result<ProcessResult, String> {
    ensure_model(&app)?;

    let bytes = tokio::task::spawn_blocking(|| -> Result<Vec<u8>, String> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| format!("Clipboard init : {e}"))?;

        let img_data = clipboard
            .get_image()
            .map_err(|e| format!("Pas d'image dans le presse-papier : {e}"))?;

        let rgba = image::RgbaImage::from_raw(
            img_data.width as u32,
            img_data.height as u32,
            img_data.bytes.into_owned(),
        )
        .ok_or_else(|| "Buffer clipboard invalide".to_string())?;

        let dyn_img = image::DynamicImage::ImageRgba8(rgba);
        encode_png(&dyn_img).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Data URL de l'original pour le split-preview côté "Avant"
    let original_data_url = format!("data:image/png;base64,{}", STANDARD.encode(&bytes));

    // Mémorise les bytes originaux pour retraitement si le fond change
    {
        let mut store = clipboard_store().lock().unwrap_or_else(|e| e.into_inner());
        *store = Some(bytes.clone());
    }

    let img = load_image_from_bytes(&bytes).map_err(|e| e.to_string())?;
    let mask = ml_engine::run_inference(&img).map_err(|e| e.to_string())?;
    let result_data_url = apply_mask(&img, &mask, &options.background)
        .and_then(|r| encode_base64_png(&r))
        .map_err(|e| e.to_string())?;

    Ok(ProcessResult { original_data_url, result_data_url })
}

/// Retraite l'image clipboard mémorisée avec un nouveau fond (sans relire le presse-papier).
#[tauri::command]
pub async fn reprocess_clipboard_image(
    app: AppHandle,
    options: ProcessOptions,
) -> Result<String, String> {
    ensure_model(&app)?;

    let bytes = {
        let store = clipboard_store().lock().unwrap_or_else(|e| e.into_inner());
        store.clone().ok_or_else(|| "Aucune image clipboard mémorisée".to_string())?
    };

    let img = load_image_from_bytes(&bytes).map_err(|e| e.to_string())?;
    let mask = ml_engine::run_inference(&img).map_err(|e| e.to_string())?;
    let result = apply_mask(&img, &mask, &options.background).map_err(|e| e.to_string())?;

    encode_base64_png(&result).map_err(|e| e.to_string())
}

/// Libère la mémoire du cache clipboard (appelé au reset).
#[tauri::command]
pub async fn clear_clipboard_cache() -> Result<(), String> {
    let mut store = clipboard_store().lock().unwrap_or_else(|e| e.into_inner());
    *store = None;
    Ok(())
}

/// Copie un résultat PNG (base64 data URL) dans le presse-papier.
#[tauri::command]
pub async fn copy_result_to_clipboard(data_url: String) -> Result<(), String> {
    let b64 = data_url
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&data_url);

    let png_bytes = STANDARD.decode(b64).map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&png_bytes).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| format!("Clipboard init : {e}"))?;

        let img_data = arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: std::borrow::Cow::Owned(rgba.into_raw()),
        };
        clipboard.set_image(img_data).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Vérifie que le chemin de destination n'est pas dans une zone système protégée.
/// Canonicalise le chemin pour prévenir les attaques par symlink/jonction/chemin UNC.
fn is_safe_save_path(path: &Path) -> Result<(), String> {
    // Bloquer les chemins UNC (\\server\share) avant canonicalisation
    let raw = path.to_string_lossy();
    if raw.starts_with("\\\\") || raw.starts_with("//") {
        return Err("Chemin UNC refusé : écriture sur des partages réseau interdite".to_string());
    }

    // Bloquer les séquences de traversal dans le chemin brut
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Chemin refusé : séquence '..' interdite".to_string());
        }
    }

    // Résoudre le chemin absolu via le répertoire parent (le fichier n'existe pas encore)
    let canonical_dir = path
        .parent()
        .ok_or_else(|| "Chemin de destination invalide : pas de répertoire parent".to_string())?
        .canonicalize()
        .map_err(|e| format!("Répertoire de destination inaccessible : {e}"))?;

    let canonical_str = canonical_dir.to_string_lossy().to_lowercase();

    let forbidden_prefixes = [
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata",
        "c:\\system",
        "c:\\users\\all users",
    ];

    if forbidden_prefixes.iter().any(|prefix| canonical_str.starts_with(prefix)) {
        return Err(format!(
            "Chemin refusé : écriture interdite dans une zone système protégée ({})",
            canonical_dir.display()
        ));
    }

    Ok(())
}

/// Sauvegarde un résultat PNG (base64 data URL) vers un fichier.
/// Écriture directe des bytes — pas de double décodage/réencodage.
#[tauri::command]
pub async fn save_result_to_file(data_url: String, dest_path: String) -> Result<(), String> {
    let dest = Path::new(&dest_path);

    is_safe_save_path(dest)?;

    let b64 = data_url
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&data_url);

    let png_bytes = STANDARD.decode(b64).map_err(|e| e.to_string())?;

    std::fs::write(dest, &png_bytes).map_err(|e| e.to_string())
}

/// Sanitise un composant de nom de fichier : supprime les caractères dangereux Windows/POSIX.
fn sanitize_filename(name: &str) -> String {
    let s = name
        .replace('\0', "")
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('|', "_")
        .replace("..", "");

    let s = s.trim().trim_matches('.').to_string();
    if s.is_empty() { "output".to_string() } else { s }
}

/// Sauvegarde plusieurs résultats dans un dossier (écriture directe, sans double décodage).
#[tauri::command]
pub async fn save_batch_to_folder(
    items: Vec<(String, String)>, // (nom_fichier, data_url)
    folder: String,
) -> Result<(), String> {
    let folder_path = PathBuf::from(&folder);

    // Valider le dossier de destination avec canonicalisation
    // Le dossier doit exister ou être créable dans une zone non-système
    std::fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;
    let canonical_folder = folder_path
        .canonicalize()
        .map_err(|e| format!("Dossier de destination inaccessible : {e}"))?;

    let folder_str = canonical_folder.to_string_lossy().to_lowercase();
    let forbidden_prefixes = [
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata",
        "c:\\system",
    ];
    if forbidden_prefixes.iter().any(|prefix| folder_str.starts_with(prefix)) {
        return Err(format!(
            "Dossier refusé : écriture interdite dans une zone système protégée ({})",
            canonical_folder.display()
        ));
    }

    for (name, data_url) in items {
        // Sanitiser le nom de fichier pour éviter path traversal dans le nom
        let stem = sanitize_filename(
            &PathBuf::from(&name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
                .to_string()
        );

        let b64 = data_url
            .strip_prefix("data:image/png;base64,")
            .unwrap_or(&data_url);
        let png_bytes = STANDARD.decode(b64).map_err(|e| e.to_string())?;

        let dest = canonical_folder.join(format!("{stem}_nobg.png"));
        std::fs::write(&dest, &png_bytes).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Vérifie que le modèle est présent.
#[tauri::command]
pub async fn check_model(app: AppHandle) -> Result<String, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?;

    let model_path = resource_dir.join("model.onnx");
    if model_path.exists() {
        Ok(model_path.to_string_lossy().to_string())
    } else {
        Err("Modèle RMBG-1.4 introuvable. Placez model.onnx dans resources/".to_string())
    }
}

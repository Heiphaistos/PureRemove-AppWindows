# Release Notes — PureRemove

> Historique des versions de PureRemove, application de suppression d'arrière-plan par IA.
> Format : [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/) — versionnage sémantique.

---

## v1.2.1 — Mai 2026 — Correctifs sécurité

### Sécurité

- **Validation de chemin dans `save_result_to_file`** — nouvelle fonction `is_safe_save_path()` en Rust qui bloque toute tentative d'écriture vers `C:\Windows`, `C:\Program Files`, `C:\Program Files (x86)` et `C:\System`. Le chemin passé depuis le frontend est systématiquement vérifié avant toute opération I/O.
- **0 vulnérabilités npm** — mise à jour de `vite`, `rollup`, `picomatch` et `postcss` via `npm audit fix`. Les 18 vulnérabilités signalées par GitHub Dependabot (7 high, 8 moderate, 3 low) ont été résorbées. Toutes concernaient des devDependencies (outils de build).

### Corrections

- `build.bat` : correction de la chaîne de version affichée (`v1.1.0` → `v1.2.0`) dans le titre, le bandeau et l'entrée de log.

---

## v1.2.0 — Mai 2026

### Nouveautés

- **Retraitement instantané du fond** — changer le mode de fond (Transparent / Blanc / Noir / Custom) re-traite l'image immédiatement sans la relire depuis le disque, grâce au cache en mémoire (`CLIPBOARD_ORIGINAL` pour le presse-papier, `singleSourceRef` pour les fichiers).
- **Support SVG** — les fichiers `.svg` sont rastérisés à 2048px minimum via `resvg` avant traitement IA. L'aspect ratio est préservé, la résolution est plafonnée à 8192px.
- **Protection VRAM (smart downscale)** — les images dépassant 4096px sur un côté sont automatiquement réduites avant inférence, évitant les erreurs de mémoire sur les grandes images.
- **Copie presse-papier du résultat** — le résultat PNG peut être copié directement dans le presse-papier via `arboard` (bouton "Copier" dans l'aperçu).
- **Color picker couleur personnalisée** — quatrième option de fond avec sélecteur de couleur HTML natif, valeur HEX affichée, persistée en localStorage.
- **Upversion ort** — passage de `ort 2.0.0-rc.11` à `ort 2.0.0-rc.12`.
- **Support format étendu** — ajout de TGA, PNM, PBM, PGM, PPM, HDR, FF, QOI.

### Corrections

- Dé-multiplication alpha correcte lors du rendu SVG (tiny_skia utilise RGBA prémultiplié — conversion explicite pour `image-rs`).
- `isValidBackground()` côté TypeScript : validation runtime stricte des payloads JSON venant du localStorage, évitant les crashs sur données corrompues.
- Gestion du montage/démontage React dans `DropZone` — `unlisten` appelé immédiatement si le composant est démonté avant la résolution de la promise Tauri.
- Propagation correcte de l'état `isProcessing` lors du retraitement fond pour éviter les interactions utilisateur pendant le calcul.

### Améliorations techniques

- **Parallélisation rayon** sur `apply_mask()` et `blur_mask()` — gain 3-4x sur les images > 2K.
- **Singleton ONNX session** via `OnceLock<Mutex<Session>>` — le modèle n'est chargé qu'une seule fois au premier traitement, puis réutilisé.
- **Prévisualisation sans double-décodage** — `original_preview_data_url()` lit les bytes natifs pour les formats web (PNG, JPEG, WebP, GIF, BMP, ICO, SVG) et ne ré-encode en PNG que pour les formats exotiques.
- **Sauvegarde batch sans double-décodage** — `save_batch_to_folder()` et `save_result_to_file()` écrivent directement les bytes PNG décodés depuis la data URL.
- **Version injectée via Vite define** — `__APP_VERSION__` lu depuis `package.json` au build, affiché dans le header sans duplication.
- CSP Tauri renforcée : `img-src` restreinte à `data:`, `asset.localhost` et `https://asset.localhost`.
- Profile release optimisé : `opt-level=3`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`.

### Notes techniques

- La cible de build est `x86_64-pc-windows-msvc` (64-bit Windows uniquement).
- Le modèle `model.onnx` n'est pas inclus dans l'installeur — voir README pour le placement.
- Les DLLs ONNX Runtime (`onnxruntime.dll`, `onnxruntime_providers_shared.dll`) sont bundlées dans `src-tauri/lib/` et copiées par Tauri lors du bundle.

---

## v1.1.0 — Avril 2026

### Nouveautés

- **Mode batch** — traitement de plusieurs images simultanément avec événements de progression temps réel (`batch-progress`).
- **Support presse-papier** — lecture d'image depuis le presse-papier via `Ctrl+V` (arboard).
- **Split-preview interactif** — curseur glissant Avant/Après avec support souris et tactile.
- **Sauvegarde dossier batch** — export de toutes les images traitées dans un dossier choisi via dialog natif.
- **Fond noir** — ajout du mode de fond `BackgroundColor::Black`.
- **Bandeau modèle manquant** — détection et affichage d'une alerte si `model.onnx` est absent au démarrage (`check_model` command).
- **Toast d'erreur global** — notification 7 secondes en bas de l'écran pour toutes les erreurs utilisateur.
- **Scripts dev.bat / build.bat** — automatisation du workflow de développement et de build.

### Corrections

- Correction du mutex poisoné sur la session ONNX : utilisation de `unwrap_or_else(|e| e.into_inner())` au lieu de `unwrap()`.
- Correction du décodage base64 : `strip_prefix("data:image/png;base64,")` avant décodage dans `copy_result_to_clipboard` et `save_result_to_file`.
- Correction de l'état `isProcessing` non réinitialisé en cas d'erreur de traitement batch.

### Améliorations

- Normalisation des extensions de fichiers en minuscules dans `is_supported()` et `isImagePath()` (front).
- `generateId()` migré vers `crypto.randomUUID()` (API native, zéro dépendance externe).
- Couleur custom persistée dans `localStorage` (clé `pureremove-custom-color`).
- Fond de sortie persisté dans `localStorage` (clé `pureremove-background`).
- Mise à jour de `ort` vers `2.0.0-rc.11`.
- Tokio limité à `rt-multi-thread` (suppression de `full` — réduction de la taille binaire).

---

## v1.0.0 — Février/Mars 2026 — Version initiale

### Fonctionnalités

- Suppression d'arrière-plan par IA locale via modèle RMBG-1.4 (BRIA).
- Interface Tauri v2 + React 18 + TypeScript + Tailwind CSS v3.
- Inférence ONNX Runtime (`ort 2.0.0-rc.x`) : tenseur CHW [1, 3, 1024, 1024], normalisation pixel/255-0.5.
- Glisser-déposer natif via `getCurrentWebview().onDragDropEvent()`.
- Ouverture de fichiers via dialog Tauri (`tauri-plugin-dialog`).
- Fond de sortie : Transparent et Blanc.
- Sauvegarde PNG via dialog natif.
- Thème dark avec damier de transparence en CSS.
- Format d'entrée : PNG, JPG/JPEG, WebP, BMP, GIF, TIFF/TIF, ICO.
- Backend Rust modulaire : `commands.rs`, `ml_engine.rs`, `image_processor.rs`.
- Blur Gaussian 3×3 sur le masque alpha pour adoucir les contours.
- Profil release optimisé (LTO, strip, panic=abort).

---

## Roadmap (non engagée)

- [ ] Support GPU via ONNX Runtime CUDA/DirectML provider
- [ ] Traitement batch parallèle (multi-thread par image)
- [ ] Export WEBP en plus de PNG
- [ ] Annulation d'un traitement en cours
- [ ] Glissement de dossier entier (récursif)
- [ ] Historique des dernières images traitées
- [ ] Mode sombre / clair configurable
- [ ] Indicateur de qualité du masque (confiance IA)

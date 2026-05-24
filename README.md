# PureRemove — Suppression d'arrière-plan par IA

![Version](https://img.shields.io/badge/version-1.2.1-blueviolet?style=flat-square)
![Plateforme](https://img.shields.io/badge/plateforme-Windows%2010%2F11-0078D4?style=flat-square&logo=windows)
![Stack](https://img.shields.io/badge/stack-Tauri%20v2%20·%20Rust%20·%20React%20·%20ONNX-orange?style=flat-square)
![Licence](https://img.shields.io/badge/licence-MIT-green?style=flat-square)

**PureRemove** est une application de bureau Windows qui supprime l'arrière-plan de vos images en quelques secondes, entièrement hors-ligne, grâce au modèle d'IA **RMBG-1.4** de BRIA. Aucune donnée n'est envoyée sur Internet — le traitement est 100% local.

---

## Fonctionnalités

- **Traitement image unique** — chargement par glisser-déposer, clic ou parcours fichier
- **Traitement en lot (batch)** — déposez plusieurs images simultanément, barre de progression en temps réel
- **Presse-papier** — collez directement une image avec `Ctrl+V`, résultat immédiat
- **Aperçu avant/après interactif** — curseur glissant pour comparer l'original et le résultat
- **4 modes de fond de sortie** — Transparent (PNG avec canal alpha), Blanc, Noir, ou couleur personnalisée via color picker
- **Retraitement instantané** — changer le fond re-traite l'image sans la relire (cache en mémoire)
- **Export flexible** — sauvegarde image unique ou lot entier dans un dossier, copie dans le presse-papier
- **18 formats d'entrée supportés** — PNG, JPG, WEBP, SVG, BMP, GIF, TIFF, ICO, TGA, PNM, PBM, PGM, PPM, HDR, FF, QOI
- **SVG vectoriel** — rastérisé à 2048px avant traitement via `resvg`
- **Protection mémoire** — downscale automatique des images > 4096px avant inférence
- **Interface sombre** — thème dark natif, damier de transparence intégré
- **Préférences persistées** — le fond de sortie et la couleur custom sont mémorisés entre les sessions
- **100% hors-ligne** — aucune connexion réseau requise après installation

---

## Captures d'écran

> Les captures d'écran seront ajoutées lors de la première release officielle.
>
> Pour contribuer des captures : placez-les dans `docs/screenshots/` et ouvrez une PR.

---

## Prérequis

- **Windows 10 version 1803** ou supérieur (x64)
- **WebView2 Runtime** (généralement pré-installé sur Windows 11 — sinon téléchargeable sur microsoft.com)
- **Modèle RMBG-1.4** (`model.onnx`) — voir section Installation

---

## Installation (release binaire)

### 1. Télécharger la release

Rendez-vous sur la page [Releases GitHub](https://github.com/heiphaistos44-crypto/PureRemove/releases) et téléchargez :

- **`PureRemove_x64-setup.exe`** — installeur NSIS (recommandé)
- ou le **ZIP portable** si disponible

### 2. Obtenir le modèle RMBG-1.4

Le modèle IA n'est **pas inclus** dans l'installeur en raison de sa taille (~178 MB).

1. Rendez-vous sur [huggingface.co/briaai/RMBG-1.4](https://huggingface.co/briaai/RMBG-1.4)
2. Téléchargez le fichier **`model.onnx`**
3. Placez-le dans le dossier `resources/` de l'application :
   - Après installation NSIS : `%AppData%\com.pureremove.desktop\resources\` (chemin variable selon Tauri)
   - En mode portable : placez `model.onnx` dans le même dossier que l'exécutable sous `resources/`

> Si le modèle est absent au démarrage, l'application affiche une bannière d'avertissement et désactive le traitement jusqu'à ce que le fichier soit détecté.

---

## Build depuis les sources

### Prérequis développeur

| Outil | Version minimale | Source |
|-------|-----------------|--------|
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| npm | 9+ | inclus avec Node.js |
| Rust (rustup) | stable (1.77+) | [rustup.rs](https://rustup.rs) |
| Cargo | inclus avec Rust | — |
| WebView2 Runtime | — | [microsoft.com](https://developer.microsoft.com/microsoft-edge/webview2/) |

### Étapes

```bash
# 1. Cloner le dépôt
git clone https://github.com/heiphaistos44-crypto/PureRemove.git
cd PureRemove

# 2. Placer le modèle IA (obligatoire)
# Télécharger model.onnx depuis huggingface.co/briaai/RMBG-1.4
# Copier dans :
cp model.onnx src-tauri/resources/model.onnx

# 3. Installer les dépendances frontend
npm install

# 4. Mode développement (rechargement à chaud)
npm run tauri dev
# ou double-cliquer sur dev.bat

# 5. Build de production (installeur NSIS x64)
npm run tauri build
# ou double-cliquer sur build.bat
```

### Artefacts de build

Après un build réussi, les artefacts se trouvent dans :

```
src-tauri/target/x86_64-pc-windows-msvc/release/
├── pure-remove.exe                       # Exécutable portable
└── bundle/nsis/
    └── PureRemove_1.2.0_x64-setup.exe   # Installeur NSIS
```

---

## Architecture

```
PureRemove/
├── src/                        # Frontend React/TypeScript
│   ├── App.tsx                 # Orchestrateur principal — état, routing UI
│   ├── components/
│   │   ├── DropZone.tsx        # Zone glisser-déposer + listener Tauri drag-drop
│   │   ├── SplitPreview.tsx    # Aperçu avant/après avec curseur interactif
│   │   ├── BatchList.tsx       # Liste de traitement en lot avec progression
│   │   └── OutputOptions.tsx   # Sélecteur de fond de sortie + color picker
│   ├── lib/utils.ts            # cn(), generateId(), hexToRgb()
│   ├── types/index.ts          # Types TypeScript partagés
│   └── styles/globals.css      # Thème dark + damier transparence
│
└── src-tauri/                  # Backend Rust
    ├── src/
    │   ├── main.rs             # Point d'entrée Windows (no_console en release)
    │   ├── lib.rs              # Tauri builder + register des commandes
    │   ├── commands.rs         # 9 commandes Tauri exposées au frontend
    │   ├── ml_engine.rs        # Inférence ONNX — session singleton thread-safe
    │   └── image_processor.rs  # Chargement, masque alpha, SVG, encodage PNG
    ├── resources/
    │   └── model.onnx          # Modèle RMBG-1.4 (non versionné — à placer manuellement)
    ├── capabilities/
    │   └── default.json        # Permissions Tauri (dialog:open, dialog:save)
    └── tauri.conf.json         # Config fenêtre, CSP, bundle NSIS
```

### Flux de traitement IA

```
Image (fichier/clipboard/batch)
    │
    ▼
[image_processor] load_image()
    ├── Formats exotiques → rasterize_svg() (resvg)
    └── Images > 4096px → smart_downscale()
    │
    ▼
[ml_engine] run_inference()
    ├── Resize → 1024×1024 (Lanczos3)
    ├── Normalisation : pixel/255 - 0.5
    ├── Tenseur CHW [1, 3, 1024, 1024]
    ├── Inférence RMBG-1.4 (ort session singleton)
    └── Post-traitement → GrayImage (masque alpha 0-255)
    │
    ▼
[image_processor] apply_mask()
    ├── blur_mask() — Gaussian 3×3 (adoucissement des contours)
    ├── Composition pixel-par-pixel (parallélisé rayon)
    └── Fond : Transparent | Blanc | Noir | Custom RGB
    │
    ▼
encode_base64_png() → data URL → Frontend React
```

---

## Stack Technique

| Composant | Technologie | Version |
|-----------|-------------|---------|
| Framework desktop | Tauri | v2 |
| Backend | Rust | stable 1.77+ |
| Frontend | React | 18.3 |
| Langage frontend | TypeScript | 5.7 |
| Build frontend | Vite | 6.0 |
| Style | Tailwind CSS | v3.4 |
| Inférence IA | ONNX Runtime (ort) | 2.0.0-rc.12 |
| Modèle IA | RMBG-1.4 (BRIA) | — |
| Traitement image | image-rs | 0.25 |
| SVG → bitmap | resvg | 0.x |
| Parallélisme | rayon | 1.x |
| Presse-papier | arboard | 3.x |
| Encodage | base64 | 0.22 |
| Dialog fichier | tauri-plugin-dialog | 2 |
| Gestion d'erreurs | anyhow | 1.x |
| Async runtime | tokio (rt-multi-thread) | 1.x |

---

## Configuration

### Fenêtre

Configurée dans `src-tauri/tauri.conf.json` :

| Paramètre | Valeur |
|-----------|--------|
| Largeur par défaut | 1280px |
| Hauteur par défaut | 820px |
| Largeur minimale | 900px |
| Hauteur minimale | 600px |
| Redimensionnable | Oui |
| Centré au démarrage | Oui |

### Sécurité (CSP)

```
default-src 'self'
img-src 'self' data: https://asset.localhost asset://localhost
style-src 'self' 'unsafe-inline'
script-src 'self'
```

### Permissions Tauri

L'application utilise uniquement les permissions minimales :

- `core:default` — IPC Tauri de base
- `core:event:default` — événements (batch-progress)
- `core:window:default` — gestion fenêtre
- `dialog:allow-open` — sélecteur de fichiers/dossiers
- `dialog:allow-save` — boîte de dialogue sauvegarde

Aucun accès au filesystem arbitraire — les chemins de sauvegarde sont toujours choisis par l'utilisateur via une boîte de dialogue native.

### Préférences utilisateur

Deux clés localStorage persistent entre les sessions :

| Clé | Contenu |
|-----|---------|
| `pureremove-background` | Fond de sortie sélectionné (JSON) |
| `pureremove-custom-color` | Dernière couleur custom choisie (hex) |

---

## Problèmes connus / Limitations

### Limitations techniques

- **Windows uniquement** — Tauri v2 avec cible `x86_64-pc-windows-msvc`. Pas de support Linux/macOS dans la configuration actuelle.
- **Traitement séquentiel en batch** — les images sont traitées une par une (le moteur ONNX tient le mutex de session). Pour de gros lots (50+ images), la progression peut être lente.
- **Modèle non bundlé** — `model.onnx` (~178 MB) n'est pas inclus dans l'installeur. L'utilisateur doit le placer manuellement.
- **Images > 4096px** — downscalées automatiquement avant inférence. La résolution du résultat est donc plafonnée à 4096px sur le grand côté.
- **GIF animés** — seule la première frame est traitée (limitation image-rs).
- **Session ONNX bloquante** — l'inférence bloque le thread courant via `Mutex<Session>`. Pas d'annulation possible en cours de traitement.

### Points d'attention sécurité

- La commande `save_result_to_file` accepte un chemin arbitraire passé depuis le frontend. Dans la configuration actuelle, ce chemin est toujours fourni par `dialog::save()` (natif), mais la validation côté Rust ne vérifie pas que le chemin est hors de zones système critiques.
- Le cache clipboard (`CLIPBOARD_ORIGINAL`) stocke les bytes de l'image en mémoire statique jusqu'à `clear_clipboard_cache()`. Un reset explicite est nécessaire pour libérer la mémoire.
- `build.bat` affiche `v1.1.0` en titre et dans le log alors que la version réelle est `1.2.0` dans `package.json` et `Cargo.toml` — incohérence mineure à corriger.

---

## Licence

Ce projet est distribué sous licence **MIT**.

Le modèle RMBG-1.4 est distribué sous sa propre licence par [BRIA AI](https://huggingface.co/briaai/RMBG-1.4) — vérifiez les conditions d'utilisation commerciale sur HuggingFace.

ONNX Runtime est distribué par Microsoft sous licence MIT. Les avis de tiers sont listés dans `src-tauri/ThirdPartyNotices.txt`.

---

## Liens utiles

- [Modèle RMBG-1.4 — HuggingFace](https://huggingface.co/briaai/RMBG-1.4)
- [ONNX Runtime](https://onnxruntime.ai/)
- [Tauri v2 Documentation](https://tauri.app/)
- [Releases GitHub](https://github.com/heiphaistos44-crypto/PureRemove/releases)

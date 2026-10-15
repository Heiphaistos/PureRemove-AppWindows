// Assemble latest.json (manifeste de mise à jour Tauri) après un build Windows signé.
//   $env:TAURI_SIGNING_PRIVATE_KEY_PATH="D:\MDP\pureremove-updater.key"; npm run tauri build
//   node scripts/make-latest-json.mjs [dossier-nsis] [fichier-notes.md]
// Joindre à la release GitHub : l'installeur *-setup.exe, son .sig et latest.json.
// (Pas de workflow Windows : le build embarque model.onnx, 170 Mo en Git LFS, et la signature
// de code est locale.)
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const dir = process.argv[2] ?? "src-tauri/target/release/bundle/nsis";
const notes = process.argv[3] ? readFileSync(process.argv[3], "utf8").trim() : `PureRemove ${version}`;
const exe = `PureRemove_${version}_x64-setup.exe`;
const sigPath = join(dir, `${exe}.sig`);
if (!existsSync(sigPath)) throw new Error(`Signature introuvable : ${sigPath} (build non signé ?)`);

const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(sigPath, "utf8").trim(),
      url: `https://github.com/Heiphaistos/PureRemove-AppWindows/releases/download/v${version}/${exe}`,
    },
  },
};
const out = join(dir, "latest.json");
writeFileSync(out, JSON.stringify(manifest, null, 2));
console.log(`écrit ${out}`);

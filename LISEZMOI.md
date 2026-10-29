# Audio Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/README.md)

[![crates.io](https://img.shields.io/crates/v/audio-helper-rs.svg)](https://crates.io/crates/audio-helper-rs) [![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Réécriture en Rust de [`audio-helper`](https://github.com/warith-harchaoui/audio-helper). Même promesse, des utilitaires audio au niveau fichier au-dessus de `ffmpeg`/`ffprobe` — pas un portage ligne à ligne du code Python, du Rust idiomatique de bout en bout.

## Périmètre

| Fonction | Ce qu'elle fait |
|---|---|
| `is_valid_audio_file(path) -> bool` | Vérification via `ffprobe` : le fichier contient-il au moins un flux audio ? |
| `get_audio_duration(path) -> Result<f64>` | Durée en secondes, via `ffprobe`. |
| `convert(input, output, &ConvertOptions) -> Result<()>` | Réencode à une fréquence d'échantillonnage / un nombre de canaux / un codec donnés. `overwrite: false` saute le travail si une sortie valide existe déjà. |
| `load(path, &LoadOptions) -> Result<PcmBuffer>` | Décode en mémoire en PCM `f32` (`Vec<f32>` + fréquence + canaux) — mono par défaut, rééchantillonnage optionnel. Fonctionne sur toute entrée lisible par `ffmpeg`, y compris la piste audio d'un conteneur vidéo. |
| `save(&PcmBuffer, path) -> Result<()>` | Réécrit du PCM dans un fichier, l'inverse de `load`. Un `.wav` est écrit directement (sans `ffmpeg`) ; toute autre extension passe par `convert`, donc le codec suit l'extension. Les échantillons hors de `-1.0..=1.0` sont écrêtés, pas repliés. |
| `extract_chunk(input, start, end, output) -> Result<()>` | Extrait une tranche `[start, end)` (secondes). |
| `generate_silence(duration, output, sample_rate) -> Result<()>` | Écrit un fichier silencieux d'une durée donnée ; vérifie ensuite que le résultat est réellement silencieux. |
| `concatenate(inputs, output) -> Result<()>` | Concatène des fichiers bout à bout via le filtre `concat` de `ffmpeg` (fonctionne même avec des codecs différents, contrairement au démuxeur `concat`). |
| `mix_room_tone(input, output, &RoomToneOptions) -> Result<()>` | Mélange un bruit de fond coloré à faible niveau sous une piste de parole — astuce classique de post-production pour que les silences d'un montage ne sonnent pas artificiellement morts. |
| `split_regularly(input, output_dir, chunk_seconds, ext, suffix) -> Result<Vec<PathBuf>>` | Découpe un fichier en tronçons de durée fixe. Un reliquat de moins d'une seconde est abandonné, sauf s'il s'agit du seul tronçon. |

```rust
use audio_helper_rs::{get_audio_duration, silence::generate_silence};

let path = std::env::temp_dir().join("example.wav");
generate_silence(1.0, &path, 16_000)?;
println!("{} secondes", get_audio_duration(&path)?);
```

### Non porté

- **`separate_sources`** (séparation de sources Demucs) : nécessite une vraie pile ML (`torch`/`torchaudio`), disproportionnée par rapport à la promesse de ce crate. Même choix que `youtube-helper-rs` qui ne réimplémente pas `yt-dlp` lui-même : une v0.1 qui délègue à un processus Python/Demucs, plutôt qu'une réimplémentation Rust native, serait l'étape naturelle suivante si le besoin se confirme.
- **`sound_resemblance`** (score de similarité par MFCC) : du DSP spécialisé (bancs de filtres mel, MFCC) qui mérite son propre crate plutôt que d'être greffé sur des utilitaires de niveau fichier.
- **Aller-retour `load`/`save` via `torch.Tensor`/`np.ndarray`** : ce crate renvoie un simple `Vec<f32>`, sans dépendance à une bibliothèque de tenseurs pour un crate bibliothèque. L'aller-retour lui-même existe : `save` est l'inverse de `load` (ajouté en 0.2.0).

## Comment ça marche

![Comment audio-helper-rs fonctionne : un fichier audio en entrée, ffmpeg piloté depuis un Rust typé, des fichiers et du PCM en sortie](https://raw.githubusercontent.com/warith-harchaoui/audio-helper-rs/master/assets/diagrams/comment-ca-marche.png)

## Installation

Nécessite `ffmpeg` et `ffprobe` sur le `PATH` (macOS : `brew install ffmpeg`) ainsi que Rust 1.88 ou plus récent.

```toml
[dependencies]
audio-helper-rs = "0.2"
```

## Ligne de commande

`cargo install audio-helper-rs` installe aussi un binaire `audio-helper-rs` — nommé d'après la
crate et non d'après la commande `audio-helper` du jumeau Python, pour que les deux
cohabitent sur un même `PATH` sans que l'un masque l'autre en silence.

```bash
audio-helper-rs probe recording.m4a
audio-helper-rs convert recording.m4a speech.wav --sample-rate 16000 --channels 1
audio-helper-rs split speech.wav -o pieces --seconds 30
```

En cas de succès, le résultat sur la sortie standard et un code de retour `0` ;
en cas d'échec, une ligne sur la sortie d'erreur préfixée `audio-helper-rs: ` et un
code `1`. Toutes les options et tous les modes d'échec :
[`skills/audio-helper-rs/references/cli-reference.md`](skills/audio-helper-rs/references/cli-reference.md).

## Skill d'agent

`skills/audio-helper-rs/` empaquette cette crate en **Claude Skill** et en **skill
OpenCode**, pour qu'un agent la découvre et l'utilise pour le compte de
l'utilisateur. L'installation pour les deux hôtes est dans
[`skills/README.md`](skills/README.md).

Son catalogue de déclencheurs est imposé, pas décoratif :
[`TRIGGERS.md`](TRIGGERS.md) est le catalogue destiné aux humains,
`references/triggers.md` en est le miroir destiné à l'agent, et
`tests/skill.rs` fait échouer `cargo test` — donc le hook de pré-push — si les
deux divergent, si la `description` du skill dépasse le plafond publié de 1024
caractères, si son `name` cesse de correspondre à son dossier ou s'il pointe
vers une référence qui n'existe plus.

## État du projet

- `cargo build` : OK, aucun avertissement.
- `cargo test` : **26 tests unitaires + 1 doctest passés**, 0 échec — chaque test qui en a besoin lance un vrai sous-processus `ffmpeg`/`ffprobe` sur un fixture WAV généré via `hound`, rien n'est simulé. Sans `ffmpeg` installé, ces tests sont sautés avec un message explicite au lieu d'échouer ; en CI (`$CI` défini) ils échouent bruyamment, car là un `ffmpeg` absent signifie que le workflow est cassé.
- `cargo clippy --all-targets` : OK, aucun avertissement.
- **Couverture de code mesurée** (`cargo llvm-cov`, remesurée le 2026-10-02 pour la 0.2.0) : **91,19 % de lignes couvertes** (795 lignes, 70 non couvertes), 87,06 % de fonctions, 91,41 % de régions. Détail par fichier :

  | Fichier | Lignes couvertes |
  |---|---|
  | `chunk.rs` | 90,00 % |
  | `concat.rs` | 86,79 % |
  | `convert.rs` | 94,12 % |
  | `ffmpeg.rs` | 82,14 % (la branche `MissingBinary` demande que `ffmpeg` soit réellement absent du `PATH`, soit précisément le cas que la suite de tests saute désormais au lieu d'échouer) |
  | `pcm.rs` | 97,85 % |
  | `probe.rs` | 83,04 % (cas limites du JSON `ffprobe`, un flux sans `duration` du tout, difficiles à fixturer sans fichier corrompu) |
  | `roomtone.rs` | 91,04 % |
  | `silence.rs` | 88,10 % |
  | `split.rs` | 95,00 % |

  Pour relancer la mesure :

  ```bash
  # une seule fois : outils LLVM (ce projet utilise Homebrew LLVM, pas rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # résumé texte
  cargo llvm-cov --html                # rapport HTML dans target/llvm-cov/html/index.html
  ```

## Vérifications avant de pousser

```bash
scripts/install-hooks.sh   # une fois par clone : installe le garde-fou pre-push ci-dessous
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
scripts/check-fresh-resolve.sh   # compile sans Cargo.lock, comme un consommateur en aval
```

Le dernier existe parce que les trois autres et la CI, compilent tous contre le
`Cargo.lock` commité — que personne n'utilise en aval du crate publié. Un intervalle de
dépendance devenu mauvais reste vert ici tout en cassant chaque `cargo add` / `cargo
install` neuf et `cargo publish --dry-run` ne le voit pas non plus (il vérifie avec le
même lock). La CI lance ce contrôle une fois, sous Linux.

## Projets liés

Fait partie du même socle d'outils locaux que [`audio-helper`](https://github.com/warith-harchaoui/audio-helper) (Python), [`podcast-helper-rs`](https://github.com/warith-harchaoui/podcast-helper-rs), [`capture-helper-rs`](https://github.com/warith-harchaoui/capture-helper-rs), [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) et la suite [AI Helpers](https://github.com/warith-harchaoui/ai-helpers). Réécriture indépendante, pas une liaison (*binding*).

## Licence

BSD-3-Clause, voir [LICENSE](LICENSE).

## Auteur

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)

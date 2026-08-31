# Audio Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/audio-helper-rs/blob/main/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/audio-helper-rs/blob/main/README.md)

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Réécriture en Rust de [`audio-helper`](https://github.com/warith-harchaoui/audio-helper). Même promesse, des utilitaires audio au niveau fichier au-dessus de `ffmpeg`/`ffprobe` — pas un portage ligne à ligne du code Python, du Rust idiomatique de bout en bout.

## Périmètre v0.1

| Fonction | Ce qu'elle fait |
|---|---|
| `is_valid_audio_file(path) -> bool` | Vérification via `ffprobe` : le fichier contient-il au moins un flux audio ? |
| `get_audio_duration(path) -> Result<f64>` | Durée en secondes, via `ffprobe`. |
| `convert(input, output, &ConvertOptions) -> Result<()>` | Réencode à une fréquence d'échantillonnage / un nombre de canaux / un codec donnés. `overwrite: false` saute le travail si une sortie valide existe déjà. |
| `load(path, &LoadOptions) -> Result<PcmBuffer>` | Décode en mémoire en PCM `f32` (`Vec<f32>` + fréquence + canaux) — mono par défaut, rééchantillonnage optionnel. Fonctionne sur toute entrée lisible par `ffmpeg`, y compris la piste audio d'un conteneur vidéo. |
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
- **Aller-retour `load`/`save` via `torch.Tensor`/`np.ndarray`** : ce crate renvoie un simple `Vec<f32>`, sans dépendance à une bibliothèque de tenseurs pour un crate bibliothèque. Réécrire un `Vec<f32>` dans un fichier revient à passer par `convert` sur un WAV écrit via [`hound`](https://crates.io/crates/hound), déjà une dépendance directe.

## Installation

Nécessite `ffmpeg` et `ffprobe` sur le `PATH` (macOS : `brew install ffmpeg`).

```toml
[dependencies]
audio-helper-rs = { git = "https://github.com/warith-harchaoui/audio-helper-rs" }
```

## État du projet

- `cargo build` : OK, aucun avertissement.
- `cargo test` : **20 tests unitaires + 1 doctest passés**, 0 échec — chaque test lance un vrai sous-processus `ffmpeg`/`ffprobe` sur un fixture WAV généré via `hound`, rien n'est simulé.
- `cargo clippy --all-targets` : OK, aucun avertissement.
- **Couverture de code mesurée** (`cargo llvm-cov`) : **88,76 % de lignes couvertes** (623 lignes, 70 non couvertes), 83,08 % de fonctions, 89,90 % de régions. Détail par fichier :

  | Fichier | Lignes couvertes |
  |---|---|
  | `chunk.rs` | 89,47 % |
  | `concat.rs` | 86,54 % |
  | `convert.rs` | 93,98 % |
  | `ffmpeg.rs` | 81,48 % (la branche `MissingBinary` demande que `ffmpeg` soit réellement absent du `PATH`, non reproduit sur une machine de CI normale) |
  | `pcm.rs` | 90,91 % |
  | `probe.rs` | 82,88 % (cas limites du JSON `ffprobe`, un flux sans `duration` du tout, difficiles à fixturer sans fichier corrompu) |
  | `roomtone.rs` | 90,77 % |
  | `silence.rs` | 87,80 % |
  | `split.rs` | 94,83 % |

  Pour relancer la mesure :

  ```bash
  # une seule fois : outils LLVM (ce projet utilise Homebrew LLVM, pas rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # résumé texte
  cargo llvm-cov --html                # rapport HTML dans target/llvm-cov/html/index.html
  ```

## Projets liés

Fait partie du même socle d'outils locaux que [`audio-helper`](https://github.com/warith-harchaoui/audio-helper) (Python), [`podcast-helper-rs`](https://github.com/warith-harchaoui/podcast-helper-rs), [`capture-helper-rs`](https://github.com/warith-harchaoui/capture-helper-rs), [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs), et la suite [AI Helpers](https://github.com/warith-harchaoui/ai-helpers). Réécriture indépendante, pas une liaison (*binding*).

## Licence

BSD-3-Clause, voir [LICENSE](LICENSE).

## Auteur

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)

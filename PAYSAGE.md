# Paysage

🇫🇷 Français · [🇬🇧 LANDSCAPE.md](https://github.com/warith-harchaoui/audio-helper-rs/blob/master/LANDSCAPE.md)

Crates voisins puis concurrents dans l'espace « manipulation de fichiers audio en Rust », comparés à `audio-helper-rs`. Les notes
vont de ⭐ (1) à ⭐⭐⭐⭐⭐ (5), évaluées sur la tâche visée par `audio-helper-rs` :
le traitement audio quotidien au niveau fichier pour des pipelines Rust (sonder, convertir, charger puis écrire du PCM, couper, concaténer, générer du silence, mélanger un fond sonore, découper en tronçons réguliers). Un crate optimisé pour un tout autre usage n'est pas pénalisé : la note
reflète seulement l'adéquation à *ce* créneau.

## En un coup d'œil

<!-- TABLE:START -->
| Manipulation audio en Rust | Décodage multi-format | Encodage / conversion | Découpage / concat / silence | Fond sonore | En processus (sans sous-processus) | Erreurs typées | Arbre de dépendances léger | Ergonomie de pipeline |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **audio-helper-rs** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| symphonia | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| rodio | ⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| hound | ⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| ffmpeg-next | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐⭐ |
| rsmpeg | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ |
| video-rs | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
<!-- TABLE:END -->

## Carte de positionnement

<!-- FIGURE:START -->
Représentation 2D du tableau ci-dessus.

![Carte de positionnement](https://raw.githubusercontent.com/warith-harchaoui/audio-helper-rs/master/assets/paysage.png)

La carte est un résumé en 2D des 8 critères : à lire comme une forme, pas comme un classement. « audio-helper-rs » se situe dans le coin en haut à droite. Les axes se lisent **Horizontal : Efficacité ↔ Rigueur** et **Vertical : Simplicité ↔ Polyvalence**.

L'axe vertical est porté par la part du travail au niveau fichier qu'un crate accomplit réellement — découper, concaténer, encoder puis le fond sonore pèsent tous positivement vers le haut — et l'axe horizontal par la légèreté ainsi que la qualité du typage à l'appel, où un arbre de dépendances léger avec des erreurs typées pèsent le plus. Cela produit trois groupes plutôt qu'un classement.

La bande du haut réunit les trois seuls crates qui décodent *et* encodent : `audio-helper-rs`, `ffmpeg-next` puis `rsmpeg`. Ils se séparent de gauche à droite sur le coût d'entrée — les deux crates de liaison lient les bibliothèques C de FFmpeg et réclament les en-têtes avec une chaîne de compilation C à la construction, là où ce crate demande quatre crates purement Rust puis les binaires à l'exécution.

Le bas à droite est la pile de lecture purement Rust : `symphonia`, `video-rs`, `rodio` et, à l'extrême, `hound`. Ils sont légers puis bien typés, ce qui explique leur position à droite ; ils décodent au lieu de transformer, ce qui explique leur position basse. `symphonia` est celui qu'il faut choisir quand un sous-processus est inacceptable ; il ne sait simplement pas encoder, or l'encodage représente la moitié du travail de ce crate.
<!-- FIGURE:END -->

## D'où viennent les affirmations

Versions puis dates de publication lues sur l'API crates.io le 29 octobre 2026 :

| Crate | Version | Dernière publication |
|---|---|---|
| `symphonia` | 0.6.1 | 2026-08-13 |
| `rodio` | 0.23.0 | 2026-10-11 |
| `hound` | 3.5.1 | 2023-09-25 |
| `ffmpeg-next` | 9.0.0 | 2026-08-05 |
| `rsmpeg` | 0.18.0+ffmpeg.8.0 | 2025-08-24 |
| `video-rs` | 0.12.0 | 2026-09-09 |

Les notes de `audio-helper-rs` lui-même proviennent de ce dépôt en version 0.3.0.

- **La colonne « En processus (sans sous-processus) » est celle où ce crate obtient sa plus mauvaise note : c'est le parti pris.** `audio-helper-rs` pilote les binaires `ffmpeg` puis `ffprobe` via `std::process::Command` ; seules la lecture WAV ainsi que la génération de silence restent dans le processus, par `hound`. `symphonia` décode dans le processus et reste le bon choix quand un sous-processus est inacceptable — il ne sait simplement pas encoder, ce qui représente la moitié du travail de ce crate.

- **`ffmpeg-next` puis `rsmpeg` obtiennent 1 sur l'arbre de dépendances** car tous deux lient les bibliothèques C de FFmpeg : il leur faut les en-têtes avec une chaîne de compilation C à la construction, là où ce crate demande quatre crates purement Rust (`thiserror`, `serde_json`, `hound`, puis `clap` seulement derrière la fonctionnalité optionnelle `cli`) ainsi que les binaires à l'exécution.

- **`hound` obtient 1 au décodage** parce qu'il lit le WAV et rien d'autre — ce qui explique précisément pourquoi ce crate en dépend plutôt que de le concurrencer.

Laissé délibérément comme opinion plutôt que sourcé : les colonnes de style
*ergonomie*, puis l'ordre des lignes sous la référence.

## Idées à reprendre

Étudié plutôt que deviné : ce que chaque voisin fait mieux, puis ce que ce crate devrait changer pour l'égaler. Rien ici n'est un engagement : c'est la liste de lecture dont part la version suivante.

- **`symphonia` — un décodage dans le processus, derrière une fonctionnalité** — La plus mauvaise colonne de ce crate est « En processus (sans sous-processus) », où il obtient 2. `symphonia` décode MP3, FLAC, AAC, ALAC, Vorbis puis WAV en Rust 100 % sûr, sans aucun sous-processus. L'ajouter derrière une fonctionnalité optionnelle donnerait `load()` à un appelant qui ne peut pas lancer de processus, sans le faire payer à personne d'autre.

- **`rodio` — des décodeurs optionnels par format plutôt qu'une dépendance dure** — `rodio` gère la lecture par `cpal`, le décodage par `symphonia` par défaut, puis le WAV par `hound` — chacun interchangeable, chacun derrière une fonctionnalité. Ce crate conditionne déjà `clap` ainsi pour la CLI ; la même forme conviendrait à un moteur de décodage.

- **`video-rs` — un chemin documenté de dix lignes jusqu'à la première image** — Son argument est qu'on obtient des données exploitables en dix lignes. L'équivalent ici est `load()` et il mérite d'être l'exemple d'ouverture du README plutôt qu'une fonction enfouie dans un tableau de périmètre.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Ruky is a modular chess engine in Rust, structured as a Cargo workspace with two crates:

- `ruky/` — chess logic, neural network, search, training. Library plus several binary targets.
- `uzi/` — standalone UCI (Universal Chess Interface) protocol library. `ruky` depends on `uzi` for I/O with UCI-speaking frontends (xboard, etc.).

The longer-term goal is an AlphaZero-style engine (CNN with policy + value heads, MCTS, self-play training, network gating by match play). Today the only playable engine is `RandomEng` (random legal moves) wired through the UCI loop.

## Build, run, test

The workspace defaults to the `cuda` feature on the `ruky` crate, which requires the CUDA toolchain and a CUDA-capable GPU. On machines without CUDA, disable defaults and pick `wgpu`:

```bash
# Default (requires CUDA)
cargo build --release

# Without CUDA — use wgpu backend
cargo build --release --no-default-features --features wgpu -p ruky
```

The features are mutually exclusive at the binary level (they select `type Backend` via `#[cfg]`), so always pass exactly one of `cuda` or `wgpu` when building `ruky`'s binaries that touch `burn`.

Common commands:

```bash
# Run the full test suite (workspace)
cargo test

# Run a single test by name across the workspace
cargo test moves_from_fen1

# Run tests in a specific crate / module
cargo test -p ruky --lib ruky::tests

# Format (rustfmt.toml requires rustfmt 1.8.0+; nightly may be needed for some options)
cargo fmt

# Lint
cargo clippy --all-targets
```

Binary targets in `ruky/src/bin/` (run with `cargo run --bin <name> -- <args>`):

- `ruky` — UCI engine entry point (currently `RandomEng`). Writes logs to `ruky.log` in the working directory.
- `self_play` — runs one self-play game with MCTS + the AlphaZero net and prints per-move stats. Parameters are hard-coded in `main` (sims, max_moves, batch_size, num_workers).
- `training` — full training pipeline: self-play → train → match-gate → repeat. Required flags: `--sessions`, `--out-dir`. Logs to `training.log`.
- `play_match` — pits two checkpointed networks against each other.
- `gen_magic` — recomputes sliding-piece magic bitboards and writes them to a file (the precomputed magics in `magics.rs` were produced by this tool).
- `scratch_nn` — ad-hoc neural network experimentation.

To play against the engine via xboard, see the root `README.md` for the exact `xboard -fcp ... -fUCI` invocation.

## Architecture

### Workspace split

`uzi` knows nothing about chess rules; it is a protocol library. `ruky` provides chess logic and implements `uzi::eng::Eng` to plug into the UCI loop. Keep this separation: chess-specific types in `ruky/src/piece.rs`, `piece_move.rs`, `sq.rs` have UCI counterparts in `uzi/src/piece.rs`, `pm.rs`, `sq.rs`, and conversion lives in adapter code on the `ruky` side (e.g. `random_eng.rs`), not by leaking chess semantics into `uzi`.

### `Ruky` as the chess factory

`ruky::Ruky` (in `ruky/src/ruky.rs`) is the convenience entry point: it owns an `Arc<ChessMagics>` (precomputed sliding-piece attack tables) and hands out `Board`s seeded with that shared magic table. Anywhere you need a fresh board or to parse a FEN, go through `Ruky::new()` → `new_board()` / `from_fen(...)` so the magics are shared instead of recomputed.

### Search → Eval → Net pipeline

The AlphaZero-style stack is layered through traits so backends and search strategies are swappable:

- `search::Search` / `search::SpSearch` (`src/search.rs`) — top-level move selection. `Search` is for normal play; `SpSearch` is the self-play variant (returns visit-count-based move distributions for training).
- Implementations: `mcts::SpMcts` (single-threaded) and `mt_mcts.rs` (multithreaded). `random_search::RandomSearch` is the no-NN baseline.
- `eval::Eval` (`src/eval.rs`) — turns a `Board` (or batch of boards) into `EvalBoards` (priors + value). `AzEval<B: Backend>` is the AlphaZero implementation: it wraps the net with `tensor_encoder::AzEncoder` and `tensor_decoder::AzDecoder` to translate between `Board` and burn `Tensor`s.
- `nn::AlphaZeroNet<B: Backend>` (`src/nn.rs`) — burn `Module` implementing the AlphaZero residual tower with policy and value heads. Generic over the burn `Backend` trait; the active backend is selected at the binary level via the `cuda`/`wgpu` cargo features.
- `tree_search::TreeSearch` — the actual MCTS tree data structure, used by `SpMcts` and the multithreaded variant.

### Training loop

`trainer::Trainer<B>` (`src/trainer.rs`) drives the full AlphaZero loop: self-play game generation (`game::TrainingGameBuilder`), dataset assembly (`dataset::{GamesDataset, GamesBatcher}`), supervised training via burn's `SupervisedTraining` strategy, and gating new checkpoints by `match_games` against the current best (`game::MatchGamesBuilder`). Configure via `TrainerBuilder` — the binary in `bin/training.rs` is a thin CLI wrapper around it.

### Magic bitboards

Sliding-piece attacks use precomputed magic bitboards (`src/magics.rs`). The constants in that file are generated by `bin/gen_magic.rs`; don't hand-edit them. If you ever need to regenerate, run `cargo run --release --bin gen_magic -- <output_file>` and replace the constants from the produced file.

### Module visibility

`ruky/src/lib.rs` keeps several modules private (`ecmv`, `fen`, `piece`, `piece_move`, `piece_set`, `sq`) and re-exports only the public surface: `Board`, `BoardBuilder`, `Piece`, `PieceMove`, `Ruky`. New external APIs should be added through this re-export list, not by making modules `pub` ad hoc.

## Conventions

- Formatter is configured in `rustfmt.toml` — `max_width = 100`, `imports_granularity = "Preserve"`, `force_explicit_abi = true`. Run `cargo fmt` before committing; CI-equivalent checks rely on the file's settings.
- `#![allow(dead_code)]` is set at the crate level for both `ruky` and `uzi` because the codebase is still WIP. Don't remove it without checking that all currently-allowed dead code is either used or deleted.
- Logging goes through the `log` crate; the binaries initialize `simple_logging` to files (`ruky.log`, `training.log`). `*log` is gitignored.

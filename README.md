# Ruky

Ruky contains components used to build a chess engine written in rust. It is in a very early stage
of development - at this point it only supports random moves. The goal is for Ruky to be a modular
chess engine employing the same techniques used by AlphaZero, and to reach a grandmaster level of
play.

## Playing against Ruky

Currently, Ruky only supports playing in random mode. To play against Ruky, one must use a frontend
with support for the UCI protocol. For example, one can use xboard to play with Ruky using the
following command:

```bash
xboard -fcp /some/path/ruky -fd . -fUCI
```

The options above translate to:
* `-fcp`: The first chess program, i.e., the chess engine, which represents the absolute or relative
  path to the ruky binary.
* `-fd`: The working directory for the first chess engine.
* `-fUCI`: Specifies that the engine uses the UCI protocol. Note that xboard uses the Polyglot
  adapter for UCI, so it maybe necessary to install the adapter.

To play Ruky against itself or another engine, one can also use xboard:

```bash
xboard -fcp /some/path/ruky -fd . -fUCI -scp /another/path/another_engine -sd . -sUCI -matchMode T
```

## Moduler design

The goal is for Ruky to be a modular chess engine built from multiple pluggable components that
ideally can be used independently of Ruky. At the moment, there are only two components with
corresponding crates:

1. `ruky`: this contains the chess logic, i.e., the part that encodes the rules and representation
   of the game.
2. `uzi`: this contains a simple UCI protocol library. It provides an interface that can be
   implemented to be able to use a frontend like xboard to play against the engine.

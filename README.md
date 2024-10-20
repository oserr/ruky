# UZI

A library implementing the UCI (Universal Chess Interface) protocol for chess engines.

## Goal

The goal is to implement a library that handles the communication protocol so that someone
wanting to implement a chess engine can focus on the chess-part of the engine, rather than
the details of a communication protocol. Some of the most popular chess engines have the
UCI part of the engine baked into the rest of the chess engine, making it difficult to
re-use. The main benefit of baking the communication protocol into the rest of the engine
is that it allows for closer integration with the actual engine, but it's very feasible
carving out the part that handles the communication protocol into a separate component
without trading-off too much. Also, the UCI protocol is clunky, not well-documented, and
difficult to understand. Having a proper library relieves users from having to waste time
chasing down bits and pieces from the web to figure out how the protocol actually works.

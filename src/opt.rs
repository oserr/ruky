// This module contains the types to represent a UCI option.

// Represents all the different options that may be supported by a UCI compliant
// chess engine.
pub enum UciOpt {
    // The value in MB for memory for hash tables.
    Hash,

    // The path on the hard disk to the Nalimov compressed format. Multiple directories can be
    // concatenated with ";".
    NalimovPath(String),

    // This is the size in MB for the cache for the nalimov table bases.
    NalimovCache(OptVal),

    // This means that the engine is able to ponder.
    Ponder(bool),

    // This means that the engine has its own book which is accessed by the engine itself. If this
    // is set, the engine takes care of the opening book. If set to false, the engine should not
    // its book.
    OwnBook(bool),

    // The engine supports multi best line or k-best mode. The default value is 1.
    MultiPv(OptVal),

    // UCI_ShowCurrLine: The engine can show the current line it is calculating.
    ShowCurrLine(bool),

    // UCI_ShowRefutations: The engine can show a move and its refutations in a line.
    ShowRefutations(bool),

    // UCI_LimitStrength: The engine is able to limit its strength to a specific elo rating. This
    // should always be implemented together with "UCI_Elo".
    LimitStrength(bool),

    // UCI_Elo: The engine can limit its strengh in Elo within this interval. Should be
    // implemented together with UCI_LimitStrength.
    Elo(OptVal),

    // UCI_AnalsysMode: The engine wants to behave differently when analysing or playing a game.
    // This is set to false if the engine is playing a game.
    AnalysisMode(bool),

    // UCI_Opponent: The command can be used by the GUI to send the name, title, elo and if the
    // engine is playing a human or computer to the engine. The format of the string is:
    // - [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>, e.g.:
    // - setoption name UCI_Opponent value GM 2800 human Garry Kasparov
    // - setoption name UCI_Opponent value none none computer Shredder
    Opponent {
        title: Option<Title>,
        elo: Option<u32>,
        player_type: PlayerType,
        name: String,
    },

    // UCI_EngineAbout: The engine tells the GUI information about itself.
    About(String),

    // UCI_ShredderbasesPath: Path to folder of containing the Shredder endgame databases.
    ShredderBasesPath(String),

    // UCI_SetPositionValue: The GUI can send this to the engine to tell it to use a certain value
    // in centipawns from white's point of view if evaluating this specific position. Allowed
    // formats:
    SetPositionValue,

    // type <t>: The option has type <t>: check, spin, combo, button, string.
    Type(OptType),
}

// The type of an option.
pub enum OptType {
    Check,
    Spin,
    Combo,
    Button,
    Str,
}

// Represents either the value an option can take, or the value to set an
// option.
pub enum OptVal {
    // Used to set a value, from GUI to engine.
    Set(i64),

    // Used to tell the GUI about the values an option can take.
    Info {
        default: i64,
        min: i64,
        max: i64,
        var: Option<i64>,
    },
}

// Represents the title of the player, e.g. grand master.
pub enum Title {
    GM,
    IM,
    FM,
    WGM,
    WIM,
}

// To represent human or computer players.
pub enum PlayerType {
    Human,
    Computer,
}

// Represents the different values that can be used for UCI_SetPositionValue.
pub enum PosValueOpt {
    // <value> + <fen>
    Val { val: i32, fen: String },
    // clear + <fen>
    Clear(String),
    // clearall
    ClearAll,
}

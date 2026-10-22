mod common;

use redscript_compiler_api::TypeInterner;

const GAME: &str = "
public class Game {
  public func Two() -> Int32 = 0
  public func Swap() -> Int32 = 0
  public func Both() -> Int32 = 0
}
";

// Two wraps of Two, a replacement of Swap, and a replacement of Both under a wrap. The line of each
// annotation is given beside it.
const MODS: &str = "
@wrapMethod(Game)
public func Two() -> Int32 = wrappedMethod()

@wrapMethod(Game)
public func Two() -> Int32 = wrappedMethod()

@replaceMethod(Game)
public func Swap() -> Int32 = 1

@replaceMethod(Game)
public func Both() -> Int32 = 2

@wrapMethod(Game)
public func Both() -> Int32 = wrappedMethod()
";
const FIRST_WRAP: u32 = 2;
const SECOND_WRAP: u32 = 5;
const SWAP: u32 = 8;
const BOTH_REPLACE: u32 = 11;
const BOTH_WRAP: u32 = 14;

fn at(path: &str, line: u32) -> (String, u32) {
    (path.to_string(), line)
}

// Each function records the source it was compiled from. A layer of a wrapped method must record
// the wrap whose body it holds, and a replaced method its replacement, as under 0.5.31; a reader
// of the bundle cannot tell otherwise which file a layer comes from (fork defect 11).
#[test]
fn layers_and_replacements_record_their_own_sources() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle(GAME, interner);
    common::compile_file("mod.reds", MODS, &mut bundle, interner);

    let game_line = |name: &str| GAME.lines().position(|l| l.contains(name)).unwrap() as u32 + 1;
    let sources = |prefix| {
        common::function_sources(&bundle, "Game", prefix)
            .into_iter()
            .map(|(name, path, line)| (name, (path, line)))
            .collect::<Vec<_>>()
    };
    let named = |name: &str, (path, line): (&str, u32)| (name.to_string(), at(path, line));

    // The method holds the outermost wrap, the one compiled last.
    assert_eq!(
        sources("Two"),
        [
            named("Two$wrapper0;", ("mod.reds", FIRST_WRAP)),
            named("Two$wrapper1;", ("game.script", game_line("Two"))),
            named("Two;", ("mod.reds", SECOND_WRAP)),
        ]
    );
    assert_eq!(sources("Swap"), [named("Swap;", ("mod.reds", SWAP))]);
    assert_eq!(
        sources("Both"),
        [
            named("Both$wrapper0;", ("mod.reds", BOTH_REPLACE)),
            named("Both;", ("mod.reds", BOTH_WRAP)),
        ]
    );
}

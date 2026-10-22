mod common;

use redscript_compiler_api::TypeInterner;

const GAME: &str = "
public class Game {
  public func Name() -> String = \"game\"
  public func Take(x: Int32) -> Int32 = x
  public cb func OnInit() -> Bool = true
}
";

// The minimal bundle has none of the game's native operators, so the layers only forward.
const WRAPS: &str = "
@wrapMethod(Game)
public func Name() -> String = wrappedMethod()

@wrapMethod(Game)
public func Name() -> String = wrappedMethod()

@wrapMethod(Game)
public func Take(x: Int32) -> Int32 = wrappedMethod(x)

@wrapMethod(Game)
public cb func OnInit() -> Bool = wrappedMethod()
";

// The game takes a function's short name from the part of its full name before the `;`. A layer
// below a wrapped method must have a short name of its own: one that repeats the method's makes
// a lookup by the short name, such as CET's, pick among the layers (fork defect 9).
#[test]
fn layers_below_a_wrapped_method_have_short_names_of_their_own() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle(GAME, interner);
    common::compile(WRAPS, &mut bundle, interner);

    let names = common::function_names(&bundle, "Game");
    assert_eq!(
        names,
        [
            "Name$wrapper0;",
            "Name$wrapper1;",
            "Name;",
            "OnInit",
            "OnInit$wrapper0",
            "Take$wrapper0;Int32",
            "Take;Int32",
        ]
    );
    for method in ["Name", "Take", "OnInit"] {
        let sharing = names
            .iter()
            .filter(|name| name.split(';').next() == Some(method))
            .count();
        assert_eq!(sharing, 1, "{method}: {names:?}");
    }
}

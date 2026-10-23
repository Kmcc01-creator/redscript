mod common;

use redscript_compiler_api::TypeInterner;
use redscript_io::IndexedDefinition;

const GAME: &str = "
public class Game {
  public func Fill(out actions: array<wref<Game>>, const context: script_ref<wref<Game>>) {}
}

public func Take(s: String) -> Int32 = 1
public func Take(s: script_ref<String>) -> Int32 = 2
public func OperatorAssignAdd(out a: Int32, b: Int32) -> Int32 = b
";

// The game's names leave out `ref` and `wref` at every level, keep `script_ref`, and write `Out`
// only for an operator: `GetQuestActions;array<DeviceAction>script_ref<GetActionsContext>`,
// `OperatorAssignAdd;OutInt32Int32;Int32`. Spelled otherwise, overloads on `T` and
// `script_ref<T>` shared a name, and an override of a game method with a `script_ref` or `out`
// parameter was named apart from it (fork defect 22). A method can't be overloaded so: the frontend
// compares parameter types with references stripped.
#[test]
fn function_names_are_spelled_as_the_game_spells_them() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let bundle = common::game_bundle(GAME, interner);

    assert_eq!(
        common::function_names(&bundle, "Game"),
        ["Fill;array<Game>script_ref<Game>"]
    );

    let mut free: Vec<_> = bundle
        .indexed_definitions()
        .filter_map(|def| match def {
            IndexedDefinition::Function(_, func)
                if func.class().is_none()
                    && ["Take;", "OperatorAssignAdd;"]
                        .iter()
                        .any(|p| bundle[func.name()].starts_with(p)) =>
            {
                Some(bundle[func.name()].to_string())
            }
            _ => None,
        })
        .collect();
    free.sort();
    assert_eq!(
        free,
        [
            "OperatorAssignAdd;OutInt32Int32",
            "Take;String",
            "Take;script_ref<String>",
        ]
    );
}

mod common;

use redscript_compiler_api::{Diagnostic, TypeInterner};

const GAME: &str = "
public class Game {
  public func Name() -> Int32 = 0
  public func Other() -> Int32 = 0
}
";

// Two replacements of Name, and one of Other. Only one replacement of a method can take
// effect, so the second of Name's must be reported (fork defect 10); 0.5.31 warns of it too.
const REPLACEMENTS: &str = "
@replaceMethod(Game)
public func Name() -> Int32 = 1

@replaceMethod(Game)
public func Name() -> Int32 = 2

@replaceMethod(Game)
public func Other() -> Int32 = 3
";

#[test]
fn a_second_replacement_of_a_method_is_reported() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle(GAME, interner);
    let diagnostics = common::compile(REPLACEMENTS, &mut bundle, interner);
    // Replacements change bodies in place: the class keeps its two functions.
    assert_eq!(common::function_names(&bundle, "Game"), ["Name;", "Other;"]);

    let conflicts = diagnostics
        .iter()
        .filter(|d| d.code() == "REPLACE_METHOD_CONFLICT")
        .collect::<Vec<_>>();
    assert_eq!(conflicts.len(), 1, "{diagnostics:?}");
    assert!(
        matches!(conflicts[0], Diagnostic::ReplaceMethodConflict(_)),
        "{conflicts:?}"
    );
    assert!(!conflicts[0].is_error());
    // Reported at the second replacement of Name, the one that takes effect.
    let span = conflicts[0].span();
    let second = REPLACEMENTS.find("Int32 = 2").unwrap();
    let first = REPLACEMENTS.find("Int32 = 1").unwrap();
    assert!(
        (span.start as usize) > first && (span.start as usize) < second,
        "{span:?}"
    );
}

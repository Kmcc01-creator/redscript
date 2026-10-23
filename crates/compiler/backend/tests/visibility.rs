mod common;

use redscript_compiler_api::TypeInterner;
use redscript_io::{IndexedDefinition, Visibility};

// The game's bundle draws boundaries the VM never checks (docs/visibility-map.md in the
// verifier): a private lifecycle hook its subclasses fill in, a protected extension point, and
// private state behind a public API.
const GAME: &str = "
public class System {
  private let state: Int32;
  private func OnAttach() -> Int32 = 0
  protected func Extend() -> Int32 = 0
  public func Api() -> Int32 = 0
}
";

fn visibility_of(name: &str, game: &redscript_io::ScriptBundle<'_>) -> Visibility {
    game.indexed_definitions()
        .find_map(|def| match def {
            IndexedDefinition::Function(_, f) if game[f.name()].starts_with(name) => {
                Some(f.visibility())
            }
            IndexedDefinition::Field(_, f) if game[f.name()] == *name => Some(f.visibility()),
            _ => None,
        })
        .unwrap()
}

fn codes(source: &str) -> Vec<&'static str> {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let game = common::game_bundle(GAME, interner);
    let mut codes = common::check(source, &game, interner)
        .iter()
        .map(|d| d.code())
        .collect::<Vec<_>>();
    codes.sort();
    codes
}

// Written as declared, and read back as written (fork defect 4: both ways were public).
#[test]
fn members_keep_their_declared_visibility_in_the_bundle() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let game = common::game_bundle(GAME, interner);
    assert_eq!(visibility_of("state", &game), Visibility::Private);
    assert_eq!(visibility_of("OnAttach", &game), Visibility::Private);
    assert_eq!(visibility_of("Extend", &game), Visibility::Protected);
    assert_eq!(visibility_of("Api", &game), Visibility::Public);
}

#[test]
fn game_members_are_reached_from_outside_only_through_their_api() {
    let codes = codes(
        "
func Outside(s: ref<System>) -> Int32 {
  s.Api();
  s.OnAttach();
  s.Extend();
  return s.state;
}
",
    );
    assert_eq!(
        codes,
        ["INACCESSIBLE_FIELD", "INACCESSIBLE_METHOD", "INACCESSIBLE_METHOD"]
    );
}

// As CDPR's scripts do 352 times, and never the other way.
#[test]
fn overrides_of_game_methods_may_narrow_but_not_widen() {
    let narrowed = codes(
        "
public class Mine extends System {
  private func OnAttach() -> Int32 = 1
  private func Extend() -> Int32 = 2
  protected func Api() -> Int32 = 3
}
public class Implicit extends System {
  func OnAttach() -> Int32 = this.Extend()
}
",
    );
    assert!(narrowed.is_empty(), "{narrowed:?}");

    let widened = codes(
        "
public class Mine extends System {
  public func OnAttach() -> Int32 = 1
  protected func OnAttach2() -> Int32 = 1
  public func Extend() -> Int32 = 2
}
",
    );
    assert_eq!(widened, ["MORE_VISIBLE_OVERRIDE", "MORE_VISIBLE_OVERRIDE"]);
}

// The way in when a mod needs internals: code placed in the class itself.
#[test]
fn code_added_to_a_game_class_reaches_its_private_members() {
    let codes = codes(
        "
@addMethod(System)
public func Peek() -> Int32 {
  this.OnAttach();
  return this.state;
}

@wrapMethod(System)
private func OnAttach() -> Int32 {
  wrappedMethod();
  return this.state;
}
",
    );
    assert!(codes.is_empty(), "{codes:?}");
}

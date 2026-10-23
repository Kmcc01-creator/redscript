mod common;

use redscript_compiler_api::TypeInterner;
use redscript_io::{IndexedDefinition, Instr, ScriptBundle};

// The game's IScriptable declares IsA natively; the minimal bundle's has no methods, so a base
// declares it here, as patmat.reds does, for the lowering to call if it still uses it.
const GAME: &str = "
public abstract class Scriptable extends IScriptable {
  public func IsA(name: CName) -> Bool = false
}

public class Request extends Scriptable {
  public let id: Int32;
}
";

// A cast, a destructuring and a weak cast pattern, each over a game class.
const PATTERNS: &str = "
class Mine extends Request {}

func CastPattern(x: ref<Scriptable>) -> Int32 {
  if let r as Request = x {
    return 1;
  }
  return 0;
}

func DestructurePattern(x: ref<Scriptable>) -> Int32 {
  if let Request { id } = x {
    return id;
  }
  return 0;
}

func WeakCastPattern(x: wref<Scriptable>) -> Int32 {
  switch x {
    case let r as Request:
      return 1;
    default:
      return 0;
  }
}
";

fn function_code(bundle: &ScriptBundle<'_>, name: &str) -> Vec<Instr> {
    bundle
        .indexed_definitions()
        .find_map(|def| match def {
            IndexedDefinition::Function(_, func)
                if func.class().is_none() && bundle[func.name()].starts_with(name) =>
            {
                Some(func.body().code_owned().unwrap())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{name} not found"))
}

// The engine's IsA takes a class's native name. The bundle names a native class by its script
// name, which can differ (ScriptableSystemRequest is gameScriptableSystemRequest), and the
// compiler knows no other, so a pattern that tested IsA(NameOf<T>()) never matched such a class
// in game (fork defect 13). A dynamic cast names no class: each pattern must test with one, and
// put no class name in a CName.
#[test]
fn cast_patterns_test_with_a_dynamic_cast() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle(GAME, interner);
    common::compile(PATTERNS, &mut bundle, interner);

    for (name, weak) in [
        ("CastPattern", false),
        ("DestructurePattern", false),
        ("WeakCastPattern", true),
    ] {
        let code = function_code(&bundle, name);
        let casts: Vec<_> = code
            .iter()
            .filter_map(|instr| match instr {
                Instr::DynamicCast { class, is_weak } => {
                    Some((bundle[bundle[*class].name()].to_string(), *is_weak))
                }
                _ => None,
            })
            .collect();
        assert_eq!(casts, [("Request".to_owned(), weak)], "{name}: {code:?}");

        let cnames: Vec<_> = code
            .iter()
            .filter_map(|instr| match instr {
                Instr::CNameConst(idx) => Some(bundle[*idx].to_string()),
                _ => None,
            })
            .collect();
        assert!(cnames.is_empty(), "{name}: {cnames:?}");
    }
}

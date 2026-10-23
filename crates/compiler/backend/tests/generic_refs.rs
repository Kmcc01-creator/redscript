mod common;

use redscript_compiler_api::TypeInterner;
use redscript_io::{IndexedDefinition, ScriptBundle};

// A generic function's result passed straight to a script_ref parameter, and the same through a
// local, which already inferred String.
const SOURCE: &str = "
native func Len(s: script_ref<String>) -> Int32

func Id<A>(a: A) -> A = a

func Direct() -> Int32 = Len(Id(\"abc\"))

func ThroughLocal() -> Int32 {
  let s: String = Id(\"abc\");
  return Len(s);
}
";

fn global_names(bundle: &ScriptBundle<'_>) -> Vec<String> {
    let mut names = bundle
        .indexed_definitions()
        .filter_map(|def| match def {
            IndexedDefinition::Function(_, func) if func.class().is_none() => {
                Some(bundle[func.name()].to_string())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

// A script_ref parameter takes a value by AsRef at the call. Inferred from that parameter, Id's
// type variable became script_ref<String>, and Id returned the reference, which in game read as
// an empty string (fork defect 14). Both calls must instantiate Id at String.
#[test]
fn a_script_ref_parameter_does_not_make_a_type_argument_a_reference() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle("", interner);
    common::compile(SOURCE, &mut bundle, interner);

    let ids: Vec<_> = global_names(&bundle)
        .into_iter()
        .filter(|name| name.starts_with("Id"))
        .collect();
    assert_eq!(ids.len(), 1, "{ids:?}");
    assert!(!ids[0].contains("script_ref"), "{ids:?}");
}

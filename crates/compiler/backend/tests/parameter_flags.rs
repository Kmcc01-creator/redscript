mod common;

use redscript_compiler_api::TypeInterner;
use redscript_io::IndexedDefinition;

const SOURCE: &str = "
public func Flags(plain: Int32, out written: Int32, opt optional: Int32, const fixed: script_ref<String>) {}
";

// Loading reads a parameter's optional, out and const flags; output wrote optional and out only,
// so a new function's `const` parameter lost its flag (fork defect 21). Each flag a source can
// declare must reach the bundle.
#[test]
fn parameter_flags_reach_the_bundle() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let mut bundle = common::game_bundle("", interner);
    common::compile(SOURCE, &mut bundle, interner);

    let func = bundle
        .indexed_definitions()
        .find_map(|def| match def {
            IndexedDefinition::Function(_, func) if bundle[func.name()].starts_with("Flags;") => {
                Some(func)
            }
            _ => None,
        })
        .unwrap();
    let flags: Vec<_> = func
        .parameters()
        .iter()
        .map(|&param| {
            let param = &bundle[param];
            let flags = param.flags();
            (
                bundle[param.name()].to_string(),
                flags.is_out(),
                flags.is_optional(),
                flags.is_const(),
            )
        })
        .collect();
    assert_eq!(
        flags,
        [
            ("plain".into(), false, false, false),
            ("written".into(), true, false, false),
            ("optional".into(), false, true, false),
            ("fixed".into(), false, false, true),
        ]
    );
}

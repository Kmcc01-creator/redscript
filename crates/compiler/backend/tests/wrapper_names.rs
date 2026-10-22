use redscript_compiler_api::ast::SourceMap;
use redscript_compiler_api::{
    CompileErrorReporter, Diagnostics, SourceMapExt, TypeFlagRegistry, TypeInterner,
};
use redscript_compiler_backend::CompilationInputs;
use redscript_compiler_frontend::infer_from_sources;
use redscript_io::{Class, ClassFlags, IndexedDefinition, ScriptBundle, Visibility};

// Compiled first, so that the second compilation finds these as the game's classes, which
// annotations may target.
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
    let interner = TypeInterner::default();
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("IScriptable");
    bundle.define(Class::new(name, Visibility::Public, ClassFlags::default()));

    compile(GAME, &mut bundle, &interner);
    // Read back from bytes, as a bundle on disk is: loading takes its names as borrowed.
    let bytes: &'static [u8] = bundle.into_writeable().to_bytes().unwrap().leak();
    let mut bundle = ScriptBundle::from_bytes(bytes).unwrap();
    compile(WRAPS, &mut bundle, &interner);

    let mut names = bundle
        .indexed_definitions()
        .filter_map(|def| match def {
            IndexedDefinition::Function(_, func)
                if func
                    .class()
                    .is_some_and(|class| &bundle[bundle[class].name()] == "Game") =>
            {
                Some(bundle[func.name()].to_string())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    names.sort();
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

fn compile<'ctx>(source: &str, bundle: &mut ScriptBundle<'ctx>, interner: &'ctx TypeInterner) {
    let sources: &'ctx SourceMap = Box::leak(Box::new(SourceMap::new()));
    sources.push_back("test.reds", source);
    sources.populate_boot_lib();

    let (symbols, mappings) =
        CompilationInputs::load(bundle, interner, &TypeFlagRegistry::default())
            .unwrap()
            .into_inner();
    let mut reporter = CompileErrorReporter::default();
    let (unit, symbols) = infer_from_sources(sources, symbols, &mut reporter, interner);
    let diagnostics = reporter.into_reported();
    assert_eq!(
        diagnostics.iter().filter(|d| d.is_error()).count(),
        0,
        "{}",
        Diagnostics::from(diagnostics)
    );
    mappings
        .into_monomorphizer(sources)
        .monomorphize(&unit, &symbols, bundle)
        .unwrap();
}

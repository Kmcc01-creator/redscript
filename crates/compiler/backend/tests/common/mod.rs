use redscript_compiler_api::ast::SourceMap;
use redscript_compiler_api::{
    CompileErrorReporter, Diagnostic, Diagnostics, SourceMapExt, TypeFlagRegistry, TypeInterner,
};
use redscript_compiler_backend::CompilationInputs;
use redscript_compiler_frontend::infer_from_sources;
use redscript_io::{Class, ClassFlags, IndexedDefinition, ScriptBundle, Visibility};

/// A bundle holding `game`, compiled and read back from bytes, so that a later compilation finds
/// its classes as the game's, which annotations may target. Loading takes a bundle's names as
/// borrowed, as they are in one read from disk.
pub fn game_bundle(game: &str, interner: &'static TypeInterner) -> ScriptBundle<'static> {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("IScriptable");
    bundle.define(Class::new(name, Visibility::Public, ClassFlags::default()));
    compile(game, &mut bundle, interner);
    let bytes: &'static [u8] = bundle.into_writeable().to_bytes().unwrap().leak();
    ScriptBundle::from_bytes(bytes).unwrap()
}

/// Compiles `source` into `bundle` and returns its diagnostics, which must hold no error.
pub fn compile(
    source: &str,
    bundle: &mut ScriptBundle<'static>,
    interner: &'static TypeInterner,
) -> Vec<Diagnostic<'static>> {
    let sources: &'static SourceMap = Box::leak(Box::new(SourceMap::new()));
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
    diagnostics
}

/// The full names of a class's functions, sorted.
pub fn function_names(bundle: &ScriptBundle<'_>, class: &str) -> Vec<String> {
    let mut names = bundle
        .indexed_definitions()
        .filter_map(|def| match def {
            IndexedDefinition::Function(_, func)
                if func
                    .class()
                    .is_some_and(|c| bundle[bundle[c].name()] == *class) =>
            {
                Some(bundle[func.name()].to_string())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

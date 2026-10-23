// Each test crate uses only some of these helpers.
#![allow(dead_code)]

use redscript_compiler_api::ast::SourceMap;
use redscript_compiler_api::{
    CompileErrorReporter, Diagnostic, Diagnostics, SourceMapExt, TypeFlagRegistry, TypeInterner,
};
use redscript_compiler_backend::CompilationInputs;
use redscript_compiler_frontend::infer_from_sources;
use redscript_io::{Class, ClassFlags, IndexedDefinition, ScriptBundle, Visibility};

/// A bundle holding `game`, compiled from `game.script` and read back from bytes, so that a later
/// compilation finds its classes as the game's, which annotations may target. Loading takes a
/// bundle's names as borrowed, as they are in one read from disk.
pub fn game_bundle(game: &str, interner: &'static TypeInterner) -> ScriptBundle<'static> {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("IScriptable");
    bundle.define(Class::new(name, Visibility::Public, ClassFlags::default()));
    compile_file("game.script", game, &mut bundle, interner);
    let bytes: &'static [u8] = bundle.into_writeable().to_bytes().unwrap().leak();
    ScriptBundle::from_bytes(bytes).unwrap()
}

/// Compiles `source`, as `test.reds`, into `bundle` and returns its diagnostics, which must hold
/// no error.
pub fn compile(
    source: &str,
    bundle: &mut ScriptBundle<'static>,
    interner: &'static TypeInterner,
) -> Vec<Diagnostic<'static>> {
    compile_file("test.reds", source, bundle, interner)
}

/// Compiles `source`, as the file `path`, into `bundle` and returns its diagnostics, which must
/// hold no error.
pub fn compile_file(
    path: &str,
    source: &str,
    bundle: &mut ScriptBundle<'static>,
    interner: &'static TypeInterner,
) -> Vec<Diagnostic<'static>> {
    let sources: &'static SourceMap = Box::leak(Box::new(SourceMap::new()));
    sources.push_back(path, source);
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

/// Checks `source`, as `test.reds`, against `bundle` and returns every diagnostic, errors
/// included. Nothing is written to the bundle.
pub fn check(
    source: &str,
    bundle: &ScriptBundle<'static>,
    interner: &'static TypeInterner,
) -> Vec<Diagnostic<'static>> {
    let sources: &'static SourceMap = Box::leak(Box::new(SourceMap::new()));
    sources.push_back("test.reds", source);
    sources.populate_boot_lib();

    let (symbols, _) = CompilationInputs::load(bundle, interner, &TypeFlagRegistry::default())
        .unwrap()
        .into_inner();
    let mut reporter = CompileErrorReporter::default();
    infer_from_sources(sources, symbols, &mut reporter, interner);
    reporter.into_reported()
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

/// The functions of a class whose full names start with `prefix`, sorted, each with the path and
/// line of its source.
pub fn function_sources(
    bundle: &ScriptBundle<'_>,
    class: &str,
    prefix: &str,
) -> Vec<(String, String, u32)> {
    let mut sources = bundle
        .indexed_definitions()
        .filter_map(|def| match def {
            IndexedDefinition::Function(_, func)
                if func
                    .class()
                    .is_some_and(|c| bundle[bundle[c].name()] == *class)
                    && bundle[func.name()].starts_with(prefix) =>
            {
                let source = func.source()?;
                Some((
                    bundle[func.name()].to_string(),
                    bundle[source.file()].path().to_string(),
                    source.line(),
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    sources.sort();
    sources
}

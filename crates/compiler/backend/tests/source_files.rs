use std::path::Path;

use redscript_compiler_api::ast::SourceMap;
use redscript_compiler_api::{CompileErrorReporter, SourceMapExt, TypeFlagRegistry, TypeInterner};
use redscript_compiler_backend::CompilationInputs;
use redscript_compiler_frontend::infer_from_sources;
use redscript_io::{Class, ClassFlags, Definition, ScriptBundle, Visibility};

// Source file definitions carry only values derived from the inputs, so identical sources compile
// to identical bundles: the path hash is FNV-1a 32 of the stored path, and the code CRC is the
// CRC32 of the file's source text.
#[test]
fn source_files_are_derived_from_inputs() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/for-loop.reds");
    let first = compile(&path);
    let second = compile(&path);
    assert!(!first.is_empty(), "no source files were emitted");
    assert_eq!(first, second, "two compilations of the same source differ");

    for (path_hash, code_crc, stored, source) in &first {
        assert_eq!(
            *path_hash,
            fnv1a32(stored.as_bytes()),
            "path hash of {stored}"
        );
        assert_eq!(
            *code_crc,
            crc32fast::hash(source.as_bytes()),
            "code CRC of {stored}"
        );
    }
}

// (path hash, code CRC, stored path, source text) of every source file in the output bundle.
fn compile(path: &Path) -> Vec<(u32, u32, String, String)> {
    let sources = SourceMap::from_files([path]).unwrap();
    sources.populate_boot_lib();

    let interner = TypeInterner::default();
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("IScriptable");
    bundle.define(Class::new(name, Visibility::Public, ClassFlags::default()));

    let (symbols, mappings) =
        CompilationInputs::load(&bundle, &interner, &TypeFlagRegistry::default())
            .unwrap()
            .into_inner();
    let mut reporter = CompileErrorReporter::default();
    let (unit, symbols) = infer_from_sources(&sources, symbols, &mut reporter, &interner);
    mappings
        .into_monomorphizer(&sources)
        .monomorphize(&unit, &symbols, &mut bundle)
        .unwrap();

    bundle
        .definitions()
        .filter_map(|def| match def {
            Definition::SourceFile(file) => Some(file),
            _ => None,
        })
        .map(|file| {
            let (_, source) = sources
                .files()
                .find(|(_, f)| f.path().to_string_lossy() == file.path())
                .expect("source file should be in the source map");
            (
                file.path_hash(),
                file.code_crc(),
                file.path().to_owned(),
                source.source().to_owned(),
            )
        })
        .collect()
}

// An independent FNV-1a 32 implementation, so the test does not share code with the compiler.
fn fnv1a32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5, |hash, &byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

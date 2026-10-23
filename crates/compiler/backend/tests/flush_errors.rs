use redscript_compiler_api::ast::SourceMap;
use redscript_compiler_api::{Compilation, FlushError, SourceMapExt, TypeInterner};
use redscript_io::{Class, ClassFlags, ScriptBundle, Visibility};

// A jump over more code than an i16 offset reaches, and a function after it.
fn too_far() -> String {
    let body = "    x = 1;\n".repeat(4000);
    format!(
        "func Far(b: Bool) -> Int32 {{\n  let x = 0;\n  if b {{\n{body}  }}\n  return x;\n}}\n\n\
         func After() -> Int32 = 7\n"
    )
}

// Monomorphization stopped at an error that had no span (a jump too distant), and flush saved
// the bundle with that function and every one after it left without code, reporting success
// (fork defect 20). The error must fail the flush, at the function, and nothing be written.
#[test]
fn a_jump_too_distant_fails_the_flush_at_its_function() {
    let mut game = ScriptBundle::default();
    let name = game.cnames_mut().add("IScriptable");
    game.define(Class::new(name, Visibility::Public, ClassFlags::default()));
    let game = game.into_writeable().to_bytes().unwrap();

    let source = too_far();
    let sources = SourceMap::new();
    sources.push_back("test.reds", &source);
    sources.populate_boot_lib();
    let interner = TypeInterner::default();
    let output =
        std::env::temp_dir().join(format!("redscript-flush-{}.redscripts", std::process::id()));
    std::fs::remove_file(&output).ok();

    let result = Compilation::builder()
        .bundle(&game)
        .sources(&sources)
        .type_interner(&interner)
        .compile()
        .unwrap()
        .flush(&output);

    let Err(FlushError::FatalErrors(diagnostics, _)) = result else {
        panic!("expected a fatal error, got {result:?}");
    };
    let errors: Vec<_> = diagnostics
        .into_iter()
        .filter(redscript_compiler_api::Diagnostic::is_error)
        .collect();
    assert_eq!(errors.len(), 1, "{errors:?}");
    let far = source.find("func Far").unwrap();
    let span = errors[0].span();
    assert!(
        (span.start as usize..span.end as usize).contains(&(far + "func ".len())),
        "{span:?} is not at Far ({far})"
    );
    assert!(!output.exists(), "the bundle was written");
}

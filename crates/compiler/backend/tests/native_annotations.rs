mod common;

use redscript_compiler_api::{Diagnostic, TypeInterner};

const GAME: &str = "
public native class Engine extends IScriptable {
  public final native func Get() -> Int32
  public final func Scripted() -> Int32 = 1
}
";

// A native method keeps its native flag, so the game never runs a wrapper's or a replacement's
// code. Accepted, either annotation reached monomorphization, which panicked on giving the native
// function a source (fork defect 17). Each must be rejected at the method, as 0.5.x does; a
// scripted method of the same class stays wrappable.
#[test]
fn annotations_on_native_methods_are_rejected() {
    let interner: &'static TypeInterner = Box::leak(Box::default());
    let game = common::game_bundle(GAME, interner);

    for source in [
        "@wrapMethod(Engine) public final func Get() -> Int32 { return wrappedMethod(); }",
        "@replaceMethod(Engine) public final func Get() -> Int32 { return 2; }",
    ] {
        let diagnostics = common::check(source, &game, interner);
        let native: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d, Diagnostic::NativeMethodAnnotation(_)))
            .collect();
        assert_eq!(native.len(), 1, "{source}: {diagnostics:?}");
        let get = source.find("Get").unwrap();
        assert_eq!(native[0].span().start as usize, get, "{source}");
    }

    let source =
        "@wrapMethod(Engine) public final func Scripted() -> Int32 { return wrappedMethod(); }";
    let diagnostics = common::check(source, &game, interner);
    assert!(
        !diagnostics.iter().any(Diagnostic::is_error),
        "{diagnostics:?}"
    );
}

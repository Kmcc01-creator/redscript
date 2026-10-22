use redscript_ast::SourceMap;
use redscript_compiler_api::{
    CompileErrorReporter, Diagnostic, SourceMapExt, Symbols, TypeInterner,
};
use redscript_compiler_frontend::infer_from_sources;

const USER_CLASS: &str = "
class UserClass {
  let x: Int32;
  func Base() -> Int32 { return 1; }
}
";

// Each annotation targets a class from another file. The diagnostics must not depend on which
// of the two files is read first, and the first must reject the annotation. (A rejected
// @wrapMethod is then compiled as a free function, whose `wrappedMethod` is unresolved.)
#[test]
fn annotations_on_user_classes_are_rejected_in_either_file_order() {
    let annotations = [
        "@addMethod(UserClass) func Added() -> Int32 { return 2; }",
        "@addField(UserClass) let y: Int32;",
        "@wrapMethod(UserClass) func Base() -> Int32 { return wrappedMethod(); }",
        "@replaceMethod(UserClass) func Base() -> Int32 { return 2; }",
    ];
    for annotation in annotations {
        let [first, second] = [true, false].map(|annotation_first| {
            diagnostics(annotation, USER_CLASS, annotation_first)
                .iter()
                .map(|diagnostic| {
                    let user_symbol = matches!(diagnostic, Diagnostic::UserSymbolAnnotation(_));
                    (diagnostic.code(), user_symbol)
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(first, second, "{annotation}");
        assert_eq!(
            first.first(),
            Some(&("INVALID_ANN_USE", true)),
            "{annotation}"
        );
    }
}

#[test]
fn annotations_on_game_classes_work_in_either_file_order() {
    let annotation = "@addMethod(IScriptable) public func Added() -> Int32 { return 2; }";
    let user = "class UserClass { func Call() -> Int32 { return this.Added(); } }";
    for annotation_first in [true, false] {
        let codes = diagnostics(annotation, user, annotation_first);
        assert!(
            codes.is_empty(),
            "annotation first: {annotation_first}: {:?}",
            codes.iter().map(Diagnostic::code).collect::<Vec<_>>()
        );
    }
}

fn diagnostics(
    annotation: &'static str,
    class: &'static str,
    annotation_first: bool,
) -> Vec<Diagnostic<'static>> {
    let sources: &'static SourceMap = Box::leak(Box::new(SourceMap::new()));
    if annotation_first {
        sources.push_back("a.reds", annotation);
        sources.push_back("b.reds", class);
    } else {
        sources.push_back("a.reds", class);
        sources.push_back("b.reds", annotation);
    }
    sources.populate_boot_lib();

    let interner: &'static TypeInterner = Box::leak(Box::default());
    let symbols = Symbols::with_default_types();
    let mut reporter = CompileErrorReporter::default();
    let (_, _) = infer_from_sources(sources, symbols, &mut reporter, interner);
    reporter.into_reported()
}

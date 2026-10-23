// Types with no default constant: the local is left as constructed (fork defect 18), and a static
// array is set element by element when its element type has one.
func Test() {
  let _: Variant;
  let _: script_ref<Int32>;
  let _: [Variant; 2];
  let _: [Int32; 3];
  let _: [Struct; 2];
  let _: Int32;
}

struct Struct {
  let a: Int32;
}

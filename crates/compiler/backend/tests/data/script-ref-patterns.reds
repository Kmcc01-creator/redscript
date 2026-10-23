// Casts over a script_ref: each must dereference before it casts (fork defect 15).
class Base {}

class Derived extends Base {
  public let x: Int32;
}

func AsCast(p: script_ref<Base>) -> Bool {
  let d = p as Derived;
  return IsDefined(d);
}

func WeakCast(p: script_ref<wref<Base>>) -> Bool {
  let d = p as Derived;
  return IsDefined(d);
}

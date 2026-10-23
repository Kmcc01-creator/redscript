// Casts and patterns over a script_ref: each must dereference before it casts, binds or reads a
// field (fork defects 15 and 16).
class Base {}

class Derived extends Base {
  public let x: Int32;
}

func AsCast(p: script_ref<Base>) -> Bool {
  let d = p as Derived;
  return IsDefined(d);
}

func CastPattern(p: script_ref<Base>) -> Bool {
  if let d as Derived = p {
    return IsDefined(d);
  }
  return false;
}

func Destructure(p: script_ref<Base>) -> Int32 {
  if let Derived { x } = p {
    return x;
  }
  return 0;
}

func WeakCast(p: script_ref<wref<Base>>) -> Bool {
  let d = p as Derived;
  return IsDefined(d);
}

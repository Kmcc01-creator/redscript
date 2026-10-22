class Base {}

class Derived extends Base {}

struct Super {
  let x: Int32;
}

struct Sub extends Super {
  let y: Int32;
}

func TakeBase(r: script_ref<Base>) {}
func TakeConstBase(const r: script_ref<Base>) {}
func TakeSuper(r: script_ref<Super>) {}

func Test() {
  let base = new Base();
  let derived = new Derived();
  TakeBase(base); // OK
  TakeBase(derived); // Error

  TakeConstBase(derived); // Error

  let sup: Super;
  let sub: Sub;
  TakeSuper(sup); // OK
  TakeSuper(sub); // OK
}

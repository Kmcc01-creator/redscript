class Base {}

class Derived extends Base {}

func TakeWeakBase(r: script_ref<wref<Base>>) {}

func Test() {
  let base = new Base();
  let derived = new Derived();
  let weakBase: wref<Base> = base;
  let weakDerived: wref<Derived> = derived;
  TakeWeakBase(weakBase); // OK
  TakeWeakBase(weakDerived); // Error
}

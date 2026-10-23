
// Overrides may narrow, as the game's scripts do, but never widen.
class Parent {
  public func Public() {}
  protected func Protected() {}
  private func Private() {}
}

class ChildWithPrivate extends Parent {
  private func Public() {} // OK
  private func Protected() {} // OK
  private func Private() {} // OK
}

class ChildWithProtected extends Parent {
  protected func Public() {} // OK
  protected func Protected() {} // OK
  protected func Private() {} // Error
}


class ChildWithPublic extends Parent {
  public func Public() {} // OK
  public func Protected() {} // Error
  public func Private() {} // Error
}

class ChildWithImplicit extends Parent {
  func Public() {} // OK
  func Protected() {} // OK
  func Private() {} // OK
}

func Testing() {
  let class = new ChildWithImplicit();
  class.Public();
  class.Protected(); // Error
  class.Private(); // Error
}

struct Vec {
  public let x: Float;
}

class Node {
  public let id: Int32;

  public final func Id() -> Int32 {
    return this.id;
  }
}

func StructRead(v: script_ref<Vec>) -> Float {
  return v.x;
}

func StructWrite(v: script_ref<Vec>, k: Float) {
  v.x = k;
}

func ClassRead(n: script_ref<Node>) -> Int32 {
  return n.id;
}

func ClassCall(n: script_ref<Node>) -> Int32 {
  return n.Id();
}

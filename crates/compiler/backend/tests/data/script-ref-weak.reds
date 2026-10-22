class Node {}

func ReadWeak(r: script_ref<wref<Node>>) -> wref<Node> {
  return r;
}

func WriteWeak(r: script_ref<wref<Node>>, w: wref<Node>) {
  r = w;
}

func PassWeak() {
  let node = new Node();
  let weak: wref<Node> = node;
  WriteWeak(weak, weak);
}

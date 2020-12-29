import wasm_init, {
  internal_init,
  internal_update,
} from "../wasm/sulafat_runtime_web.js";
import { Decoder } from "./bincode.js";
import { todo, unreachable } from "./util.js";

let root: Node | Node[];

export async function init() {
  await wasm_init("/wasm/sulafat_runtime_web_bg.wasm");
  const buffer = internal_init();
  const decoder = new Decoder(buffer.buffer);
  const node = deserializeNode(decoder);
  decoder.end();
  root = node;
  mount(node);
}

function mount(node: Node | Node[]) {
  const mountPoint = document.getElementById("mount-point");
  while (mountPoint?.childNodes.length) {
    mountPoint.firstChild?.remove();
  }
  if (node instanceof Array) {
    mountPoint?.append(...node);
  } else {
    mountPoint?.append(node);
  }
}

const NODE_SINGLE = 0;
const NODE_LIST = 1;

function deserializeNode(decoder: Decoder): Node | Node[] {
  switch (decoder.u32()) {
    case NODE_SINGLE:
      return deserializeSingle(decoder);
    case NODE_LIST:
      return deserializeList(decoder);
    default:
      unreachable();
  }
}

function deserializeList(decoder: Decoder): Node[] {
  const len = decoder.u64();
  const list: Node[] = [];
  for (let i = 0; i < len; i += 1) {
    list.push(deserializeSingle(decoder));
  }
  return list;
}

const SINGLE_TEXT = 0;
const SINGLE_ELEMENT = 1;

function deserializeSingle(decoder: Decoder): Node {
  switch (decoder.u32()) {
    case SINGLE_TEXT: {
      return document.createTextNode(decoder.string());
    }
    case SINGLE_ELEMENT: {
      return deserializeElement(decoder);
    }
    default:
      unreachable();
  }
}

const ELEMENT_DIV = 0;
const ELEMENT_SPAN = 1;

function deserializeElement(decoder: Decoder): Element {
  switch (decoder.u32()) {
    case ELEMENT_DIV: {
      const [id, children] = deserializeCommon(decoder);
      const div = document.createElementNS(
        "http://www.w3.org/1999/xhtml",
        "div"
      );
      if (id) {
        div.setAttribute("id", id);
      }
      div.append(...children);
      return div;
    }
    case ELEMENT_SPAN: {
      const [id, children] = deserializeCommon(decoder);
      const span = document.createElementNS(
        "http://www.w3.org/1999/xhtml",
        "span"
      );
      if (id) {
        span.setAttribute("id", id);
      }
      span.append(...children);
      return span;
    }
    default:
      unreachable();
  }
}

function deserializeCommon(
  decoder: Decoder
): [id: string | undefined, children: Node[]] {
  const key = decoder.optional("string");
  const id = decoder.optional("string");
  const children = deserializeList(decoder);
  return [id, children];
}

const OPTIONAL_NONE = 0;
const OPTIONAL_SOME = 1;

export function update() {
  const buffer = internal_update();
  if (buffer) {
    const node = applyNode(root, new Decoder(buffer.buffer));
    if (root != node) {
      root = node;
      mount(node);
    }
  }
}

const PATCH_NODE_REPLACE = 0;
const PATCH_NODE_SINGLE = 1;
const PATCH_NODE_LIST = 2;

function applyNode(node: Node | Node[], decoder: Decoder): Node | Node[] {
  switch (decoder.u32()) {
    case PATCH_NODE_REPLACE: {
      return deserializeNode(decoder);
    }
    case PATCH_NODE_SINGLE: {
      if (node instanceof Array) {
        throw Error("単一ノードではありません");
      }
      return applySingle(node, decoder);
    }
    case PATCH_NODE_LIST: {
      if (!(node instanceof Array)) {
        throw Error("ノードリストではありません");
      }
      return applyList(node, decoder);
    }
    default:
      unreachable();
  }
}

const PATCH_SINGLE_REPLACE = 0;
const PATCH_SINGLE_ELEMENT = 1;

function applySingle(node: Node, decoder: Decoder): Node {
  switch (decoder.u32()) {
    case PATCH_SINGLE_REPLACE:
      return deserializeSingle(decoder);
    case PATCH_SINGLE_ELEMENT:
      if (node.nodeType !== Node.ELEMENT_NODE) {
        throw Error("Elementではありません");
      }
      return applyElement(node as Element, decoder);
    default:
      unreachable();
  }
}

const PATCH_LIST_ALL = 0;
const PATCH_LIST_ENTRIES = 1;

const PATCH_LIST_OP_NOP = 0;
const PATCH_LIST_OP_MODIFY = 1;
const PATCH_LIST_OP_FROM = 2;
const PATCH_LIST_OP_FROM_MODIFY = 3;
const PATCH_LIST_OP_NEW = 4;

function applyList(list: Node[], decoder: Decoder): Node[] {
  switch (decoder.u32()) {
    case PATCH_LIST_ALL: {
      const len = decoder.u64();
      const newList: Node[] = [];
      for (let i = 0; i < len; i += 1) {
        switch (decoder.u32()) {
          case PATCH_LIST_OP_NOP:
            newList.push(list[i]);
            break;
          case PATCH_LIST_OP_MODIFY:
            newList.push(applySingle(list[i], decoder));
            break;
          case PATCH_LIST_OP_FROM: {
            const from = decoder.u64();
            newList.push(list[from]);
            break;
          }
          case PATCH_LIST_OP_FROM_MODIFY: {
            const from = decoder.u64();
            newList.push(applySingle(list[from], decoder));
            break;
          }
          case PATCH_LIST_OP_NEW:
            newList.push(deserializeSingle(decoder));
            break;
          default:
            unreachable();
        }
      }
      return newList;
    }
    case PATCH_LIST_ENTRIES: {
      const len = decoder.u64();
      const entries_len = decoder.u64();
      const newList = list.slice(0, len);
      for (let i = 0; i < entries_len; i += 1) {
        let index = decoder.u64();
        const node: Node | undefined = newList[index];
        switch (decoder.u32()) {
          case PATCH_SINGLE_ELEMENT: {
            if (node?.nodeType !== Node.ELEMENT_NODE) {
              throw Error("Elementではありません");
            }
            newList[index] = applyElement(node as Element, decoder);
            break;
          }
          case PATCH_SINGLE_REPLACE: {
            if (index >= newList.length) {
              newList.push(deserializeSingle(decoder));
            } else {
              newList[index] = deserializeSingle(decoder);
            }
            break;
          }
          default:
            unreachable();
        }
      }
      return newList;
    }
    default:
      unreachable();
  }
}

const PATCH_ELEMENT_REPLACE = 0;
const PATCH_ELEMENT_DIV = 1;
const PATCH_ELEMENT_SPAN = 2;

function applyElement(element: Element, decoder: Decoder): Element {
  switch (decoder.u32()) {
    case PATCH_ELEMENT_REPLACE:
      return deserializeElement(decoder);
    case PATCH_ELEMENT_DIV: {
      if (
        element.namespaceURI !== "http://www.w3.org/1999/xhtml" ||
        element.localName !== "div"
      ) {
        throw Error("divではありません");
      }
      return applyDiv(element as HTMLDivElement, decoder);
    }
    case PATCH_ELEMENT_SPAN: {
      if (
        element.namespaceURI !== "http://www.w3.org/1999/xhtml" ||
        element.localName !== "span"
      ) {
        throw Error("spanではありません");
      }
      return applySpan(element as HTMLSpanElement, decoder);
    }
    default:
      unreachable();
  }
}

function applyDiv(div: HTMLDivElement, decoder: Decoder): HTMLDivElement {
  if (decoder.bool()) {
    const id = decoder.optional("string");
    if (id) {
      div.setAttribute("id", id);
    } else {
      div.removeAttribute("id");
    }
  }
  if (decoder.bool()) {
    const children = applyList(Array.from(div.childNodes), decoder);
    while (div.firstChild) {
      div.removeChild(div.firstChild);
    }
    div.append(...children);
  }
  return div;
}

function applySpan(span: HTMLSpanElement, decoder: Decoder): HTMLSpanElement {
  if (decoder.bool()) {
    const id = decoder.optional("string");
    if (id) {
      span.setAttribute("id", id);
    } else {
      span.removeAttribute("id");
    }
  }
  if (decoder.bool()) {
    const children = applyList(Array.from(span.childNodes), decoder);
    while (span.firstChild) {
      span.removeChild(span.firstChild);
    }
    span.append(...children);
  }
  return span;
}

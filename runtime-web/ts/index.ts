import wasm_init, {
  internal_init,
  internal_render,
  internal_on_event,
} from "../wasm/sulafat_runtime_web.js";
import { Decoder } from "./bincode.js";
import { todo, unreachable } from "./util.js";

let root: Node | Node[];

export async function init() {
  await wasm_init("/wasm/sulafat_runtime_web_bg.wasm");
  const buffer = internal_init();
  console.log(buffer);
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

const eventHandlerMap: WeakMap<
  EventTarget,
  Record<string, EventListener>
> = new WeakMap();
function elementHandlersOf(e: EventTarget): Record<string, EventListener> {
  const item = eventHandlerMap.get(e);
  if (item) {
    return item;
  }
  const newItem = Object.create(null) as Record<string, EventListener>;
  eventHandlerMap.set(e, newItem);
  return newItem;
}
function registerEventListener(
  e: EventTarget,
  event: string,
  handler: (..._: never[]) => void
) {
  elementHandlersOf(e)[event] = handler as EventListener;
  e.addEventListener(event, handler as EventListener);
}
function unregisterEventListener(e: EventTarget, event: string) {
  const handler = elementHandlersOf(e)[event];
  if (handler) {
    e.removeEventListener(event, handler);
  }
}

function deserializeElement(decoder: Decoder): Element {
  let element: HTMLElement;
  switch (decoder.u32()) {
    case ELEMENT_DIV: {
      element = document.createElementNS("http://www.w3.org/1999/xhtml", "div");
      break;
    }
    case ELEMENT_SPAN: {
      element = document.createElementNS(
        "http://www.w3.org/1999/xhtml",
        "span"
      );
      break;
    }
    default:
      unreachable();
  }
  const [attr, children] = deserializeCommon(decoder);
  for (const a of attr) {
    switch (a[0]) {
      case ATTRIBUTE_ID:
        element.setAttribute("id", a[1]);
        break;
      case ATTRIBUTE_ON_CLICK: {
        const id = a[1];
        const buf = new Uint8Array(16);
        buf.set(id);
        registerEventListener(element, "click", () => {
          console.log(id);
          buf.set(new Uint8Array(new Uint32Array([0]).buffer), 12);
          internal_on_event(buf);
        });
        break;
      }
      case ATTRIBUTE_ON_POINTER_MOVE: {
        const id = a[1];
        const buf = new Uint8Array(32);
        buf.set(id);
        buf.set(new Uint8Array(new Uint32Array([1]).buffer), 12);
        registerEventListener(element, "pointermove", (e: PointerEvent) => {
          console.log(id);
          buf.set(new Uint8Array(new Float64Array([e.x, e.y]).buffer), 16);
          internal_on_event(buf);
        });
        break;
      }
    }
  }
  element.append(...children);
  return element;
}

function deserializeCommon(decoder: Decoder): [attr: Attr[], children: Node[]] {
  const attr = [...deserializeAttr(decoder)];
  const children = deserializeList(decoder);
  return [attr, children];
}

const ATTRIBUTE_ID = 0;
const ATTRIBUTE_ON_CLICK = 1;
const ATTRIBUTE_ON_POINTER_MOVE = 2;
type AttrTypes = {
  [ATTRIBUTE_ID]: string;
  [ATTRIBUTE_ON_CLICK]: Uint8Array;
  [ATTRIBUTE_ON_POINTER_MOVE]: Uint8Array;
};
type Attr = {
  [K in keyof AttrTypes]: [K, AttrTypes[K]];
}[keyof AttrTypes];

function* deserializeAttr(decoder: Decoder): Generator<Attr> {
  const len = decoder.u64();
  for (let i = 0; i < len; i += 1) {
    const attr: Attr[0] = decoder.u32() as Attr[0];
    switch (attr) {
      case ATTRIBUTE_ID:
        yield [attr, decoder.string()];
        break;
      case ATTRIBUTE_ON_CLICK:
        yield [attr, deserializeHandlerId(decoder)];
        break;
      case ATTRIBUTE_ON_POINTER_MOVE:
        yield [attr, deserializeHandlerId(decoder)];
        break;
      default:
        unreachable();
    }
  }
}

function deserializeHandlerId(decoder: Decoder): Uint8Array {
  return decoder.read(12);
}

export function render() {
  const buffer = internal_render();
  if (buffer) {
    console.log(buffer);
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
  return applyCommon(div, decoder);
}

function applySpan(span: HTMLSpanElement, decoder: Decoder): HTMLSpanElement {
  return applyCommon(span, decoder);
}

const PATCH_ATTRIBUTE_REMOVE = 0;
const PATCH_ATTRIBUTE_INSERT = 1;

function applyCommon<E extends Element>(element: E, decoder: Decoder): E {
  const len = decoder.u64();
  for (let i = 0; i < len; i += 1) {
    switch (decoder.u32()) {
      case PATCH_ATTRIBUTE_REMOVE:
        switch (decoder.u32()) {
          case ATTRIBUTE_ID:
            element.removeAttribute("id");
            break;
          case ATTRIBUTE_ON_CLICK:
            unregisterEventListener(element, "click");
            break;
          case ATTRIBUTE_ON_POINTER_MOVE:
            unregisterEventListener(element, "pointermove");
            break;
          default:
            unreachable();
        }
        break;
      case PATCH_ATTRIBUTE_INSERT:
        switch (decoder.u32()) {
          case ATTRIBUTE_ID:
            element.setAttribute("id", decoder.string());
            break;
          case ATTRIBUTE_ON_CLICK: {
            unregisterEventListener(element, "click");
            const id = deserializeHandlerId(decoder);
            const buf = new Uint8Array(16);
            buf.set(id);
            registerEventListener(element, "click", () => {
              console.log(id);
              buf.set(new Uint8Array(new Uint32Array([0]).buffer), 12);
              internal_on_event(buf);
            });
            break;
          }
          case ATTRIBUTE_ON_POINTER_MOVE: {
            const id = deserializeHandlerId(decoder);
            const buf = new Uint8Array(32);
            buf.set(id);
            buf.set(new Uint8Array(new Uint32Array([1]).buffer), 12);
            registerEventListener(element, "pointermove", (e: PointerEvent) => {
              console.log(id);
              buf.set(new Uint8Array(new Float64Array([e.x, e.y]).buffer), 16);
              internal_on_event(buf);
            });
            break;
          }
          default:
            unreachable();
        }
        break;
      default:
        unreachable();
    }
  }
  if (decoder.bool()) {
    const children = applyList(Array.from(element.childNodes), decoder);
    while (element.firstChild) {
      element.removeChild(element.firstChild);
    }
    element.append(...children);
  }
  return element;
}

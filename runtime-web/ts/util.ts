export function unreachable(): never {
  throw Error("unreachable!");
}

export function todo(): never {
  throw Error("under construction!");
}

export class Decoder {
  #buffer: ArrayBufferLike;
  public constructor(buffer: ArrayBufferLike) {
    this.#buffer = buffer;
  }

  public bool(): boolean {
    return !!this.i8();
  }

  public i8(): number {
    const value = new Int8Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(1);
    return value;
  }

  public u8(): number {
    const value = new Uint8Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(1);
    return value;
  }

  public i16(): number {
    const value = new Int16Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(2);
    return value;
  }

  public u16(): number {
    const value = new Uint16Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(2);
    return value;
  }

  public i32(): number {
    const value = new Int32Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(4);
    return value;
  }

  public u32(): number {
    const value = new Uint32Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(4);
    return value;
  }

  public i64(): number {
    const value = new BigInt64Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(8);
    return Number(value);
  }

  public u64(): number {
    const value = new BigUint64Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(8);
    return Number(value);
  }

  public i64n(): bigint {
    const value = new BigInt64Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(8);
    return value;
  }

  public u64n(): bigint {
    const value = new BigUint64Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(8);
    return value;
  }

  public f32(): number {
    const value = new Float32Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(4);
    return value;
  }

  public f64(): number {
    const value = new Float64Array(this.#buffer, 0, 1)[0];
    this.#buffer = this.#buffer.slice(8);
    return value;
  }

  public string(): string {
    const len = this.u64();
    const text_decoder = new TextDecoder("utf-8", { fatal: true });
    const value = text_decoder.decode(this.#buffer.slice(0, len));
    this.#buffer = this.#buffer.slice(len);
    return value;
  }

  public char(): number {
    const buf = new Uint8Array(this.#buffer);
    let value = 0xffffffff;
    if (buf[0] < 0x80) {
      value = buf[0];
      this.#buffer = this.#buffer.slice(1);
    } else if (buf[0] < 0xc0) {
      throw Error(
        `UTF-8 decoding error. Unexpected byte. 0x[${buf[0].toString(16)}]`
      );
    } else if (buf[0] < 0xe0) {
      if ((buf[1] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[1].toString(16)}]`
        );
      }
      value = ((buf[0] & 0x1f) << 6) | (buf[1] & 0x3f);
      if (value < 0x80) {
        throw Error(`UTF-8 decoding error. Invalid encoding.`);
      }
      this.#buffer = this.#buffer.slice(2);
    } else if (buf[0] < 0xf0) {
      if ((buf[1] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[1].toString(16)}]`
        );
      }
      if ((buf[2] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[2].toString(16)}]`
        );
      }
      value =
        ((buf[0] & 0x0f) << 12) | ((buf[1] & 0x3f) << 6) | (buf[2] & 0x3f);
      if (value < 0x800 || 0xd800 <= value || value < 0xe000) {
        throw Error(`UTF-8 decoding error. Invalid encoding.`);
      }
      this.#buffer = this.#buffer.slice(3);
    } else if (buf[0] < 0xf5) {
      if ((buf[1] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[1].toString(16)}]`
        );
      }
      if ((buf[2] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[2].toString(16)}]`
        );
      }
      if ((buf[3] & 0xc0) !== 0x80) {
        throw Error(
          `UTF-8 decoding error. Unexpected byte. 0x[${buf[3].toString(16)}]`
        );
      }
      value =
        ((buf[0] & 0x07) << 18) |
        ((buf[1] & 0x3f) << 12) |
        ((buf[2] & 0x3f) << 6) |
        (buf[3] & 0x3f);
      if (value < 0x010000 || 0x110000 <= value) {
        throw Error(`UTF-8 decoding error. Invalid encoding.`);
      }
      this.#buffer = this.#buffer.slice(4);
    } else {
      `UTF-8 decoding error. Unexpected byte. 0x[${buf[0].toString(16)}]`;
    }
    return value;
  }

  public optional<
    Method extends Exclude<keyof Decoder, "constructor" | "end" | "optional">
  >(
    method: Method,
    ...args: Parameters<Decoder[Method]>
  ): ReturnType<Decoder[Method]> | undefined {
    if (this.bool()) {
      return (this as Record<Method, () => unknown>)[method](
        ...(args as [])
      ) as ReturnType<Decoder[Method]>;
    }
  }

  public end() {
    if (this.#buffer.byteLength) {
      throw Error("Trailing buffer exists.");
    }
  }

  public read(n: number): Uint8Array {
    if (n < this.#buffer.byteLength) {
      n = this.#buffer.byteLength;
    }
    const array = Uint8Array.from(new Uint8Array(this.#buffer, 0, n));
    this.#buffer = this.#buffer.slice(n);
    return array;
  }
}

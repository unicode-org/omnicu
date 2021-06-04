import icu4x from "./icu4x.mjs"
import * as rtti from "./rtti.mjs"

function withEncodedString(str, fn) {
  let bytes = (new TextEncoder()).encode(str);

  let ptr = icu4x.icu4x_alloc(bytes.length);
  const buf = new Uint8Array(icu4x.memory.buffer, ptr, bytes.length);
  buf.set(bytes, 0);

  try {
    return fn(ptr, buf.length);
  } finally {
    icu4x.icu4x_free(ptr, buf.length);
  }
}

function readString(ptr, len) {
  const buf = new Uint8Array(icu4x.memory.buffer, ptr, len);
  return (new TextDecoder("utf-8")).decode(buf)
}

const fixedDecimalRegistry = new FinalizationRegistry(ptr => {
  icu4x.icu4x_fixed_decimal_destroy(ptr);
});

const ICU4XFixedDecimalMultiplyPow10Result = {
  parse: rtti.StructType({
    success: [ 0, rtti.ScalarType.bool ],
    error_code: [ 4, rtti.ScalarType.i32 ]
  }),
  size: 8
};

export class FixedDecimal {
  constructor(magnitude) {
    this.underlying = icu4x.icu4x_fixed_decimal_create(magnitude);    
    fixedDecimalRegistry.register(this, this.underlying);
  }

  multiply_pow10(pow) {
    const receiveBuffer = icu4x.icu4x_alloc(ICU4XFixedDecimalMultiplyPow10Result.size);
    icu4x.icu4x_fixed_decimal_multiply_pow10(receiveBuffer, this.underlying, pow);
    const parsed = ICU4XFixedDecimalMultiplyPow10Result.parse(icu4x.memory.buffer, receiveBuffer);
    icu4x.icu4x_free(receiveBuffer);
    return parsed;
  }

  negate() {
    icu4x.icu4x_fixed_decimal_negate(this.underlying);
  }

  write_to(writable) {
    icu4x.icu4x_fixed_decimal_write_to(this.underlying, writable.underlying);
  }
}

const bufferWritableRegistry = new FinalizationRegistry(ptr => {
  icu4x.icu4x_buffer_writeable_destroy(ptr);
});

export class BufferWritable {
  constructor() {
    this.underlying = icu4x.icu4x_buffer_writeable_create(0);    
    bufferWritableRegistry.register(this, this.underlying);
  }

  getString() {
    const outStringPtr = icu4x.icu4x_buffer_writeable_get_bytes(this.underlying);
    const outStringLen = icu4x.icu4x_buffer_writeable_len(this.underlying);
    return readString(outStringPtr, outStringLen);
  }
}
import test from 'ava';

import { FixedDecimal, BufferWritable, Locale, StaticDataProvider, FixedDecimalFormat } from "../lib/high-level.mjs"

const locale = new Locale("bn");
const dataProvider = new StaticDataProvider();
const format = new FixedDecimalFormat(locale, dataProvider, {});

test("format a simple decimal", t => {
  const decimal = new FixedDecimal(BigInt(1234));
  decimal.multiply_pow10(-2);

  const outWritable = new BufferWritable();
  format.write(decimal, outWritable);
  t.is(outWritable.getString(), "১২.৩৪");
});

test("format a long decimal", t => {
  const decimal = new FixedDecimal(BigInt(1000007));

  const outWritable = new BufferWritable();
  format.write(decimal, outWritable);
  t.is(outWritable.getString(), "১০,০০,০০৭");
});

test("format a negated, scaled decimal", t => {
  const decimal = new FixedDecimal(BigInt(1000007));
  decimal.multiply_pow10(2);
  decimal.negate();

  const outWritable = new BufferWritable();
  format.write(decimal, outWritable);
  t.is(outWritable.getString(), "-১০,০০,০০,৭০০");
});

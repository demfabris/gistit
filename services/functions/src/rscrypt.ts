import { scryptSync } from "crypto";
import assert from "assert";

type ScryptPayload = {
  hash: Buffer;
  salt: Buffer;
  params: ScryptParams;
};

type ScryptParams = {
  N: number;
  r: number;
  p: number;
};

/**
 * This functions parses the scrypt derived key in "rust" format, a.k.a 'rscrypt'.
 * Implementation follows: https://docs.rs/rust-crypto/0.2.36/src/crypto/scrypt.rs.html#318-407
 *
 * @function
 * @param {string} derivedKey
 * @return {ScryptPayload}
 */
function rscryptDerivedKeyDisassemble(derivedKey: string): ScryptPayload {
  const [__prefix, name, paramFormat, params, salt, hash, __tail] =
    derivedKey.split("$");

  // Rscrypt formatting asssertions
  assert.equal(__prefix, "", "prefix should be '$'");
  assert.equal(__tail, "", "tail should be '$'");
  assert.equal(name, "rscrypt", "name should be rscrypt");

  let scryptParams: ScryptParams;
  const paramsDecoded = Buffer.from(params, "base64");

  if (paramFormat === "0") {
    // Compact param format
    assert.equal(paramsDecoded.length, 3, "should be length 3");
    const [logN, paramR, paramP] = paramsDecoded;
    scryptParams = {
      N: 1 << Number(logN),
      r: Number(paramR),
      p: Number(paramP),
    };
  } else if (paramFormat === "1") {
    // Extended format
    assert.equal(paramsDecoded.length, 9, "should be length 9");
    const logN = paramsDecoded[0];
    const [paramR, paramP] = [
      paramsDecoded.slice(1, 4),
      paramsDecoded.slice(5),
    ];
    scryptParams = {
      N: 1 << Number(logN),
      r: Number(paramR),
      p: Number(paramP),
    };
  } else {
    throw new Error("invalid scrypt params");
  }

  return {
    hash: Buffer.from(hash, "base64"),
    salt: Buffer.from(salt, "base64"),
    params: scryptParams,
  };
}

/**
 * Compares scrypt derived key with provided secret
 *
 * @function
 * @param {string} derivedKey
 * @param {string} rawSecret
 * @return {boolean}
 */
export function rscryptCompare(derivedKey: string, rawSecret: string): boolean {
  const payload = rscryptDerivedKeyDisassemble(derivedKey);
  const lhsDerivedKey = payload.hash;
  const rhsDerivedKey = scryptSync(rawSecret, payload.salt, 32, payload.params);

  return Buffer.compare(lhsDerivedKey, rhsDerivedKey) === 0;
}

import * as __fn from "firebase-functions";
import defs from "./defs.json";

/**
 * Checks for a valid gistit hash
 * @function
 * @param {string} hash
 */
export function checkHash(hash: string): void {
  if (hash.length === defs.HASH_LENGTH) {
    switch (hash[0]) {
      case defs.HASH_P2P_PREFIX:
        __fn.logger.log("p2p");
        break;
      case defs.HASH_SERVER_PREFIX:
        __fn.logger.log("server");
        break;
      default:
        break;
    }
  } else throw Error("Invalid gistit hash format");
}

/**
 * Checks author and description char length
 * @function
 * @param {string} author
 * @param {string} description
 * @param {string} secret
 */
export function checkParamsCharLength(
  author: string,
  description: string,
  secret: string
): void {
  if (
    paramValueInRange(defs.AUTHOR_CHAR_LENGTH, author?.length) &&
    paramValueInRange(defs.DESCRIPTION_CHAR_LENGTH, description?.length) &&
    paramValueInRange(defs.SECRET_CHAR_LENGTH, secret?.length)
  )
    return;
  else throw Error("Invalid author, description or secret character length");
}

/**
 * @function
 * @param {number} size
 */
export function checkFileSize(size: number): void {
  if (paramValueInRange(defs.FILE_SIZE, size)) return;
  else throw Error("File size not allowed");
}

/**
 * Check if timestamp is between margin of error
 * If it took more than 120s to reach server we refuse it
 * @function
 * @param {number} timestamp
 * @param {number} lifespan
 */
export function checkTimeDelta(timestamp: number, lifespan: number): void {
  const serverNow = Date.now();
  const timeDelta = serverNow - Number(timestamp);

  if (Math.abs(timeDelta) > defs.TIMESTAMP_DELTA_LIMIT_MS)
    throw Error("Time delta beyond allowed limit, check your system time");

  if (paramValueInRange(defs.LIFESPAN_VALUE, lifespan)) return;
  else throw Error("Invalid lifespan parameter value");
}

interface RangeObj {
  MIN: number;
  MAX: number;
}
/**
 * @function
 * @param {RangeObj} obj
 * @param {number} value
 * @return {boolean}
 */
function paramValueInRange(obj: RangeObj, value?: number): boolean {
  if (value && value > obj.MAX && value < obj.MIN) return false;
  return true;
}

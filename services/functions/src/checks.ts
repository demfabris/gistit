import defs from "./defs.json";

/**
 * Checks for a valid gistit hash
 * @function
 * @param {string} hash
 */
export function checkHash(hash: string): void {
  if (hash.length !== defs.HASH_LENGTH)
    throw Error("Invalid gistit hash format");
}

/**
 * Checks author and description char length
 * @function
 * @param {string} author
 * @param {string} description
 */
export function checkParamsCharLength(
  author: string,
  description: string
): void {
  if (
    paramValueInRange(defs.AUTHOR_CHAR_LENGTH, author?.length) &&
    paramValueInRange(defs.DESCRIPTION_CHAR_LENGTH, description?.length)
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

import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";
import defs from "./defs.json";

__adm.initializeApp();
const db = __adm.firestore();

type GistitPayload = {
  hash: string;
  author: string;
  description: string;
  colorscheme: string;
  lifespan: number;
  timestamp: number;
  secret: string;
  gistit: {
    name: string;
    lang: string;
    data: string;
  };
};

/**
 * Checks for a valid gistit hash
 * @function
 * @param {string} hash
 */
function checkHash(hash: string) {
  if (hash.length === defs.HASH_LENGTH) {
    switch (hash[0]) {
      case defs.HASH_P2P_PREFIX:
        __fn.logger.log("p2p");
        break;
      case defs.HASH_SERVER_PREFIX:
        __fn.logger.log("server");
        break;
      default:
        throw Error("invalid gistit hash format");
    }
  }
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
function paramValueInRange(obj: RangeObj, value: number): boolean {
  if (value > obj.MAX || value < obj.MIN) return false;
  return true;
}

/**
 * Checks author and description char length
 * @function
 * @param {string} author
 * @param {string} description
 * @param {string} secret
 */
function checkParamsCharLength(
  author: string,
  description: string,
  secret: string
) {
  if (
    paramValueInRange(defs.AUTHOR_CHAR_LENGTH, author.length) &&
    paramValueInRange(defs.DESCRIPTION_CHAR_LENGTH, description.length) &&
    paramValueInRange(defs.SECRET_CHAR_LENGTH, secret.length)
  )
    return;
  else throw Error("Invalid author or description character length");
}

/**
 * Check if timestamp is between margin of error
 * If it took more than 120s to reach server we refuse it
 * @function
 * @param {number} timestamp
 * @param {number} lifespan
 */
function checkTimeDelta(timestamp: number, lifespan: number) {
  const serverNow = Date.now();
  const timeDelta = serverNow - timestamp;

  if (Math.abs(timeDelta) > defs.TIMESTAMP_DELTA_LIMIT_MS)
    throw Error("time delta beyond allowed limit, check your system time");

  if (paramValueInRange(defs.LIFESPAN_VALUE, lifespan)) return;
  else throw Error("invalid lifespan parameter value");
}

export const load = __fn.https.onRequest(async (req, res) => {
  try {
    const {
      hash,
      author,
      description,
      colorscheme,
      lifespan,
      timestamp,
      secret,
      gistit: { name, lang, data },
    } = req.body;

    checkHash(hash);
    checkParamsCharLength(author, description, secret);
    checkTimeDelta(timestamp, lifespan);

    await db.collection("gistits").doc(hash).set({
      author,
      description,
      colorscheme,
      lifespan,
      secret,
      timestamp,
      gistit: { name, lang, data },
    });
    res.send({ success: hash });
  } catch (err) {
    __fn.logger.error(err);
    res.status(400).send({ error: (err as Error).message });
  }
});

interface onChangeContext extends __fn.EventContext {
  params: {
    hash: string;
  };
}
export const createReservedData = __fn.firestore
  .document("gistits/{hash}")
  .onCreate(async (snap, context) => {
    const hash = (context as onChangeContext).params.hash;
    const { timestamp, lifespan } = snap.data() as GistitPayload;

    return db
      .collection("reserved")
      .doc(hash)
      .set({
        removeAt: timestamp + lifespan * 1000,
        reuploaded: 0,
      });
  });

export const updateReservedData = __fn.firestore
  .document("gistits/{hash}")
  .onUpdate(async (change, context) => {
    const hash = (context as onChangeContext).params.hash;
    const { timestamp, lifespan } = change.after.data() as GistitPayload;

    return db
      .collection("reserved")
      .doc(hash)
      .update({
        removeAt: timestamp + lifespan * 1000,
        reuploaded: __adm.firestore.FieldValue.increment(1),
      });
  });

export const scheduledCleanup = __fn.pubsub
  .schedule("every 30 mins")
  .onRun(async () => {
    const expiredDocuments = await db
      .collection("reserved")
      .where("removeAt", "<", Date.now())
      .get();

    expiredDocuments.forEach(async (doc) => {
      const hash = doc.id;
      await db.doc(`reserved/${hash}`).delete();
      await db.doc(`gistits/${hash}`).delete();
      __fn.logger.info("deleted hash: ", hash);
    });
    return null;
  });

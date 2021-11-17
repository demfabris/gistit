import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";
import example from "../../example.json";

const HASH_LENGTH = 33;
const HASH_P2P_PREFIX = "@";
const HASH_SERVER_PREFIX = "$";
const DESCRIPTION_CHAR_LENGTH = 100;
const AUTHOR_CHAR_LENGTH = 30;
const SECRET_CHAR_LENGTH = 30;
const TIMESTAMP_DELTA_LIMIT = 120000;
const LIFESPAN_MAX_VALUE = 3600;

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

(async () => {
  try {
    const {hash, ...rest} = example as GistitPayload;
    await db.collection("gistits").doc(hash).set(rest);
    await db.collection("__db").doc("server").set({state: "running"});
    __fn.logger.log("successfully init db");
  } catch (err) {
    __fn.logger.error(err);
  }
})();

/**
 * Checks for a valid gistit hash
 * @param {string} hash
 */
function checkHash(hash: string) {
  if (hash.length === HASH_LENGTH) {
    switch (hash[0]) {
      case HASH_P2P_PREFIX:
        __fn.logger.log("p2p");
        break;
      case HASH_SERVER_PREFIX:
        __fn.logger.log("server");
        break;
      default:
        throw Error("invalid gistit hash format");
    }
  }
}

/**
 * Checks author and description char length
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
    description.length > DESCRIPTION_CHAR_LENGTH ||
    author.length > AUTHOR_CHAR_LENGTH ||
    secret.length > SECRET_CHAR_LENGTH
  ) {
    throw Error("Invalid author or description character length");
  }
}

/**
 * Check if timestamp is between margin of error
 * If it took more than 120s to reach server we refuse it
 * @param {number} timestamp
 * @param {number} lifespan
 */
function checkTimeDelta(timestamp: number, lifespan: number) {
  const serverNow = Date.now();
  // TODO: The incoming timestamp is in seconds, we need to provide in ms
  const timeDelta = serverNow - timestamp * 1000;
  if (Math.abs(timeDelta) > TIMESTAMP_DELTA_LIMIT) {
    throw Error("time delta beyond allowed limit, check your system time");
  }
  if (lifespan > LIFESPAN_MAX_VALUE) {
    throw Error("invalid lifespan parameter value");
  }
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
      gistit: {name, lang, data},
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
      gistit: {name, lang, data},
    });
    res.send({success: hash});
  } catch (err) {
    __fn.logger.error(err);
    res.status(400).send({error: (err as Error).message});
  }
});

interface onChangeContext extends __fn.EventContext {
  params: {
    hash: string;
  };
}
export const writeReservedData = __fn.firestore
    .document("gistits/{hash}")
    .onWrite(async (change, context) => {
      const hash = (context as onChangeContext).params.hash;
      const {timestamp, lifespan} = change.after.data() as GistitPayload;

      return db
          .collection("reserved")
          .doc(hash)
          .set({
            removeAt: timestamp + lifespan,
            reupload: false,
          });
    });

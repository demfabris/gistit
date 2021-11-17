import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";
import schema from "../../schema.json";

const constants = {
  __HASH_LENGTH: 33,
  __HASH_P2P_PREFIX: "@",
  __HASH_SERVER_PREFIX: "$",
  __DESCRIPTION_CHAR_LENGTH: 100,
  __AUTHOR_CHAR_LENGTH: 30,
  __SECRET_CHAR_LENGTH: 30,
  __TIMESTAMP_DELTA_LIMIT: 120000,
  __LIFESPAN_MAX_VALUE: 3600,
};

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
    file: {
      mime: string;
      data: string;
    };
  };
};

/**
 * Compute the time when the gistit should be destroyed
 * @param {GistitPayload} payload
 * @return {number}
 */
function getTimeToRemove(payload: GistitPayload): number {
  return payload.timestamp + payload.lifespan;
}

(async () => {
  try {
    const {hash, ...rest} = schema.gistits.first;
    await db.collection("gistits").doc(hash).set(rest);
    await db.collection("reserved").doc("server").set({__state: "running"});
    await db
        .collection("toRemove")
        .doc(hash)
        .set({removeAt: getTimeToRemove(schema.gistits.first)});
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
  if (hash.length === constants.__HASH_LENGTH) {
    switch (hash[0]) {
      case constants.__HASH_P2P_PREFIX:
        __fn.logger.log("p2p");
        break;
      case constants.__HASH_SERVER_PREFIX:
        __fn.logger.log("server");
        break;
      default:
        throw Error("invalid hash");
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
    description.length > constants.__DESCRIPTION_CHAR_LENGTH ||
    author.length > constants.__AUTHOR_CHAR_LENGTH ||
    secret.length > constants.__SECRET_CHAR_LENGTH
  ) {
    throw Error("invalid author or description char length");
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
  const timeDiff = serverNow - timestamp * 1000;
  __fn.logger.log("serverNow: ", serverNow);
  __fn.logger.log("timeDifff: ", timeDiff);
  if (Math.abs(timeDiff) > constants.__TIMESTAMP_DELTA_LIMIT) {
    throw Error("invalid timestamp");
  }
  if (lifespan > constants.__LIFESPAN_MAX_VALUE) {
    throw Error("invalid lifespan");
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
      gistit: {
        name,
        lang,
        file: {mime, data},
      },
    } = req.body;
    checkHash(hash);
    checkParamsCharLength(author, description, secret);
    checkTimeDelta(timestamp, lifespan);
    await db
        .collection("gistits")
        .doc(hash)
        .set({
          author,
          description,
          colorscheme,
          lifespan,
          secret,
          timestamp,
          gistit: {name, lang, file: {mime, data}},
        });
    res.send("ok");
  } catch (err) {
    __fn.logger.error(err);
    res.status(400).send("Invalid request body");
  }
});

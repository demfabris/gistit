type TokenMap = {
  [key: string]: { [entry: string]: string };
};

export const tokenMap: TokenMap = {};

import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";

import { GistitPayload } from "./gistit";
import { checkHash, checkParamsCharLength, checkFileSize } from "./checks";

export { auth, token } from "./auth";
export {
  createReservedData,
  updateReservedData,
  scheduledCleanup,
} from "./reserved";

__adm.initializeApp();
export const db = __adm.firestore();

export const load = __fn.https.onRequest(async (req, res) => {
  try {
    const {
      hash,
      author,
      description,
      timestamp,
      inner: { name, lang, data, size },
    } = req.body as GistitPayload;

    checkHash(hash);
    checkParamsCharLength(author, description);
    checkFileSize(size);

    await db.collection("gistits").doc(hash).set({
      author,
      description,
      timestamp: timestamp.toString(),
      inner: { name, lang, data, size },
    });

    __fn.logger.info("added gistit: ", hash);
    res.send({
      success: {
        hash,
        author,
        description,
        timestamp,
        inner: { name, lang, data: { inner: "" }, size },
      },
    });
  } catch (err) {
    res.status(400).send({ error: (err as Error).message });
  }
});

type FetchPayload = {
  hash: string;
};

export const get = __fn.https.onRequest(async (req, res) => {
  try {
    const { hash } = req.body as FetchPayload;
    const gistitRef = await db.collection("gistits").doc(hash).get();
    if (!gistitRef.exists) {
      res.status(404).send({ error: "Gistit does not exist" });
      return;
    }
    const gistit = gistitRef.data();
    res.send({ success: { ...gistit, hash } });
  } catch (err) {
    res.status(400).send({ error: (err as Error).message });
  }
});

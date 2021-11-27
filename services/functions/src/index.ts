import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";
import {
  checkHash,
  checkParamsCharLength,
  checkTimeDelta,
  checkFileSize,
} from "./checks";
import { rscryptCompare } from "./rscrypt";

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
      gistit: { name, lang, data, size },
    } = req.body;

    checkHash(hash);
    checkParamsCharLength(author, description, secret);
    checkTimeDelta(timestamp, lifespan);
    checkFileSize(size);

    await db.collection("gistits").doc(hash).set({
      author,
      description,
      colorscheme,
      lifespan,
      secret,
      timestamp,
      gistit: { name, lang, data },
    });
    __fn.logger.info("added gistit: ", hash);
    res.send({ success: hash });
  } catch (err) {
    res.status(400).send({ error: (err as Error).message });
  }
});

type FetchPayload = {
  hash: string;
  secret?: string;
};

export const get = __fn.https.onRequest(async (req, res) => {
  try {
    const { hash, secret } = req.body as FetchPayload;
    if (!secret) {
      res.status(401).send({ error: "This gistit requires password" });
      return;
    }
    const gistitRef = await db.collection("gistits").doc(hash).get();
    const derivedKey = gistitRef.get("secret");

    if (rscryptCompare(derivedKey, secret)) {
      const gistit = gistitRef.data();
      res.send({ success: { ...gistit, secret } });
    }
  } catch (err) {
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
    });
    return null;
  });

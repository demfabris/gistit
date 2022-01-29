import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";
import { checkHash, checkParamsCharLength, checkFileSize } from "./checks";
import { LIFESPAN } from "./defs.json";

__adm.initializeApp();
const db = __adm.firestore();

type GistitPayload = {
  hash: string;
  author: string;
  description: string;
  timestamp: string;
  inner: {
    name: string;
    lang: string;
    data: string;
    size: number;
  };
};

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

interface onChangeContext extends __fn.EventContext {
  params: {
    hash: string;
  };
}

export const createReservedData = __fn.firestore
  .document("gistits/{hash}")
  .onCreate(async (snap, context) => {
    const hash = (context as onChangeContext).params.hash;
    const { timestamp } = snap.data() as GistitPayload;

    return db
      .collection("reserved")
      .doc(hash)
      .set({
        removeAt: timestamp + LIFESPAN,
        reuploaded: 0,
      });
  });

export const updateReservedData = __fn.firestore
  .document("gistits/{hash}")
  .onUpdate(async (change, context) => {
    const hash = (context as onChangeContext).params.hash;
    const { timestamp } = change.after.data() as GistitPayload;

    return db
      .collection("reserved")
      .doc(hash)
      .update({
        removeAt: timestamp + LIFESPAN,
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

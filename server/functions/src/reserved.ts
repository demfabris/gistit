import * as __fn from "firebase-functions";
import * as __adm from "firebase-admin";

import { db } from "./index";
import { GistitPayload } from "./gistit";
import { LIFESPAN } from "./defs.json";

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

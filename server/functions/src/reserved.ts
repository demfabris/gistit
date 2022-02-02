import * as functions from "firebase-functions";
import * as admin from "firebase-admin";

import { db } from "./index";

interface onChangeContext extends functions.EventContext {
  params: {
    hash: string;
  };
}

export const createReservedData = functions.firestore
  .document("gistits/{hash}")
  .onCreate(async (_, context) => {
    const hash = (context as onChangeContext).params.hash;

    return db
      .collection("reserved")
      .doc(hash)
      .set({
        removeAt: Date.now() + 300 * 60 * 100,
        reuploaded: 0,
      });
  });

export const updateReservedData = functions.firestore
  .document("gistits/{hash}")
  .onUpdate(async (_, context) => {
    const hash = (context as onChangeContext).params.hash;

    return db
      .collection("reserved")
      .doc(hash)
      .update({
        removeAt: Date.now() + 300 * 60 * 100,
        reuploaded: admin.firestore.FieldValue.increment(1),
      });
  });

export const gistitScheduledCleanup = functions.pubsub
  .schedule("every 500 mins")
  .onRun(async () => {
    const expiredDocuments = await db
      .collection("reserved")
      .where("removeAt", "<", Date.now())
      .get();

    expiredDocuments.forEach(async (doc) => {
      const hash = doc.id;
      functions.logger.info(`gistit cleanup: ${hash}`);
      await db.doc(`reserved/${hash}`).delete();
      await db.doc(`gistits/${hash}`).delete();
    });
    return null;
  });

import * as functions from "firebase-functions";
import fetch from "cross-fetch";
import { db } from "./index";

interface AuthPayload {
  code: string;
  state: string;
}

export const token = functions.https.onRequest(async (req, res) => {
  try {
    const { state } = req.body as AuthPayload;

    const tokenRef = await db.collection("githubToken").doc(state).get();

    if (!tokenRef.exists) {
      throw Error("state doesn't match any tokens");
    }

    const token = tokenRef.data();
    await db.collection("githubToken").doc(state).delete();

    res.status(200).send(token);
  } catch (error) {
    res.status(404).send({ error: "not found" });
  }
});

export const auth = functions.https.onRequest(async (req, res) => {
  res
    .setHeader("Access-Control-Allow-Origin", "https://gistit.vercel.app")
    .setHeader("Access-Control-Allow-Credentials", "true")
    .setHeader("Access-Control-Allow-Methods", "GET,HEAD,OPTIONS,POST,PUT")
    .setHeader(
      "Access-Control-Allow-Headers",
      "Access-Control-Allow-Headers, Origin, Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers"
    );

  try {
    const { code, state } = JSON.parse(req.body) as AuthPayload;

    if (!code || !state) {
      res.status(500).send({ error: "unexpected request" });
      return;
    }

    const clientSecret = functions.config().github.secret;
    const clientId = functions.config().github.id;
    const payload = {
      client_id: clientId,
      client_secret: clientSecret,
      code,
    };

    const response = await fetch(
      "https://github.com/login/oauth/access_token",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(payload),
      }
    );

    const data = await response.json();
    if (data?.error === "bad_verification_code") {
      res.status(500).send({ error: "expired" });
      return;
    }

    if (data?.["access_token"]) {
      await db
        .collection("githubToken")
        .doc(state)
        .set({
          ...data,
          expireAt: Date.now() + 60 * 10 * 1000,
        });

      res.status(200).send({ success: "authenticated" });
      return;
    }

    res.status(400).send({ error: "executon failed" });
  } catch (err) {
    res.end();
  }
});

export const tokenScheduledCleanup = functions.pubsub
  .schedule("every 60 mins")
  .onRun(async () => {
    const expiredTokens = await db
      .collection("githubToken")
      .where("expireAt", "<", Date.now())
      .get();

    expiredTokens.forEach(async (doc) => {
      const tokenId = doc.id;
      functions.logger.info(`token cleanup: ${tokenId}`);
      await db.doc(`githubToken/${tokenId}`).delete();
    });

    return null;
  });

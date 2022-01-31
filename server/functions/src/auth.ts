import * as __fn from "firebase-functions";
import fetch from "cross-fetch";
import { db } from "./index";

interface AuthPayload {
  code: string;
  state: string;
}

export const token = __fn.https.onRequest(async (req, res) => {
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
    __fn.logger.error(error);
    res.status(400).send({ error: "not found" });
  }
});

export const auth = __fn.https.onRequest(async (req, res) => {
  res
    .setHeader("Access-Control-Allow-Origin", "*")
    .setHeader("Access-Control-Allow-Credentials", "true")
    .setHeader("Access-Control-Allow-Methods", "GET,HEAD,OPTIONS,POST,PUT")
    .setHeader(
      "Access-Control-Allow-Headers",
      "Access-Control-Allow-Headers, Origin,Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers"
    );

  try {
    const { code, state } = JSON.parse(req.body) as AuthPayload;

    if (!code || !state) {
      res.status(500).send({ error: "unexpected request" });
      return;
    }

    const clientSecret = __fn.config().github.secret;
    const clientId = __fn.config().github.id;
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
      await db.collection("githubToken").doc(state).set(data);
      __fn.logger.info(state, data);
      res.status(200).send({ success: "authenticated" });
      return;
    }

    res.status(400).send({ error: "executon failed" });
  } catch (err) {
    __fn.logger.error("failed to execute auth", err);
    res.end();
  }
});

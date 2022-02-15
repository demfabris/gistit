import * as functions from "firebase-functions";
import * as admin from "firebase-admin";
import protobuf from "protobufjs";

export { auth, token, tokenScheduledCleanup } from "./auth";
export {
  createReservedData,
  updateReservedData,
  gistitScheduledCleanup,
} from "./reserved";

admin.initializeApp();

export const db = admin.firestore();

const GISTIT_HASH_LENGTH = 64; // md5 hash

const GISTIT_AUTHOR_MAX_CHAR_LENGTH = 50;
const GISTIT_AUTHOR_MIN_CHAR_LENGTH = 3;

const GISTIT_DESCRIPTION_MAX_CHAR_LENGTH = 100;
const GISTIT_DESCRIPTION_MIN_CHAR_LENGTH = 10;

const GISTIT_FILE_MAX_SIZE = 50_000_000; // 50kb
const GISTIT_FILE_MIN_SIZE = 20; // 20 bytes

export type GistitPayload = {
  hash: string;
  author: string;
  description: string;
  timestamp: string;
  inner: {
    name: string;
    lang: string;
    data: string;
    size: number;
  }[];
};

export const load = functions.https.onRequest(async (req, res) => {
  const proto = await protobuf.load("payload.proto");
  const Gistit = proto.lookupType("gistit.payload.Gistit");
  const payload = Gistit.decode(req.body);

  try {
    const {
      hash,
      author,
      description,
      timestamp,
      inner: [{ name, lang, size, data }],
    } = payload as unknown as GistitPayload;
    functions.logger.log(payload);

    if (hash?.length !== GISTIT_HASH_LENGTH)
      throw Error("Invalid gistit hash format");

    if (
      author &&
      (author.length > GISTIT_AUTHOR_MAX_CHAR_LENGTH ||
        author.length < GISTIT_AUTHOR_MIN_CHAR_LENGTH)
    ) {
      throw Error("Invalid author length");
    }

    if (
      description &&
      (description.length > GISTIT_DESCRIPTION_MAX_CHAR_LENGTH ||
        description.length < GISTIT_DESCRIPTION_MIN_CHAR_LENGTH)
    ) {
      throw Error("Invalid description length");
    }

    if (
      data.length > GISTIT_FILE_MAX_SIZE ||
      data.length < GISTIT_FILE_MIN_SIZE
    ) {
      throw Error("File size is not allowed");
    }

    await db.collection("gistits").doc(hash).set({
      author,
      description,
      timestamp: timestamp.toString(),
      inner: { name, lang, data, size },
    });

    functions.logger.info("added gistit: ", hash);
    const response = Gistit.encode({
      hash,
      author,
      description,
      timestamp,
      inner: [{ name, lang, data: "", size }],
    }).finish();

    res.send(response);
  } catch (err) {
    functions.logger.error(err);
    res.status(400).end();
  }
});

type FetchPayload = {
  hash: string;
};

export const get = functions.https.onRequest(async (req, res) => {
  res
    // .setHeader("Access-Control-Allow-Origin", "https://gistit.vercel.app")
    .setHeader("Access-Control-Allow-Origin", "*")
    .setHeader("Access-Control-Allow-Credentials", "true")
    .setHeader("Access-Control-Allow-Methods", "GET,HEAD,OPTIONS,POST,PUT")
    .setHeader(
      "Access-Control-Allow-Headers",
      "Access-Control-Allow-Headers, Origin, Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers"
    );

  try {
    let payload: FetchPayload = { hash: "" };

    if (req.body instanceof Object) {
      payload = req.body as FetchPayload;
    } else {
      payload = JSON.parse(req.body) as FetchPayload;
    }

    const { hash } = payload;
    functions.logger.debug(hash);

    if (hash?.length !== GISTIT_HASH_LENGTH)
      throw Error("Invalid gistit hash format");

    const gistitRef = await db.collection("gistits").doc(hash).get();

    if (!gistitRef.exists) {
      res.status(404).send({ error: "Gistit does not exist" });
      return;
    }

    const gistit = gistitRef.data();
    res.status(200).send({ success: { ...gistit, hash } });
  } catch (err) {
    res.status(400).send({ error: (err as Error).message });
  }
});

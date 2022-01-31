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
  };
};

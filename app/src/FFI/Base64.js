// URL-safe Base64 over UTF-8. Works in the browser (btoa/atob +
// TextEncoder/TextDecoder) and under Node (Buffer), so the same code is
// exercised by the unit tests (node) and the app (browser).

const hasBuffer = typeof Buffer !== "undefined";

const toUrl = (b64) =>
  b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");

const fromUrl = (s) => {
  const b64 = s.replace(/-/g, "+").replace(/_/g, "/");
  const pad = b64.length % 4;
  return pad === 0 ? b64 : b64 + "=".repeat(4 - pad);
};

export const base64urlEncode = (text) => {
  if (hasBuffer) {
    return toUrl(Buffer.from(text, "utf8").toString("base64"));
  }
  // Browser: UTF-8 bytes → binary string → btoa.
  const bytes = new TextEncoder().encode(text);
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return toUrl(btoa(bin));
};

export const base64urlDecodeImpl = (nothing) => (just) => (s) => {
  try {
    if (hasBuffer) {
      return just(Buffer.from(fromUrl(s), "base64").toString("utf8"));
    }
    const bin = atob(fromUrl(s));
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return just(new TextDecoder().decode(bytes));
  } catch (_) {
    return nothing;
  }
};

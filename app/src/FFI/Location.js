export const getHash = () => window.location.hash || "";

export const getLocationHref = () => window.location.href;

// Replace the fragment in place; no navigation / reload. Preserve the
// existing path + query.
export const setHash = (hash) => () => {
  const url = new URL(window.location.href);
  url.hash = hash;
  window.history.replaceState(null, "", url.toString());
};

export const getQueryParamsImpl = (tuple) => () => {
  const out = [];
  const params = new URLSearchParams(window.location.search);
  for (const [k, v] of params.entries()) {
    out.push(tuple(k)(v));
  }
  return out;
};

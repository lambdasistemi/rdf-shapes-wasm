// FFI for the rdf-shapes-wasm engine. `globalThis.rdfShapes` is seeded
// by bootstrap.js (initSync'd). The engine returns a plain JS object
// (serde-wasm-bindgen json_compatible) on success and throws a JS Error
// on failure; we catch the throw and pass its message to `left`.

const errText = (e) =>
  e && e.message ? String(e.message) : String(e);

export const queryImpl = (left) => (right) => (graphTtl) => (sparql) => () => {
  try {
    return right(globalThis.rdfShapes.query(graphTtl, sparql));
  } catch (e) {
    return left(errText(e));
  }
};

export const validateImpl = (left) => (right) => (dataTtl) => (shapesTtl) => () => {
  try {
    return right(globalThis.rdfShapes.validate(dataTtl, shapesTtl));
  } catch (e) {
    return left(errText(e));
  }
};

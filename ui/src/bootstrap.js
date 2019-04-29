// A dependency graph that contains any wasm must all be imported
// asynchronously.
import("./index.js")
  .catch(e => console.error("Error importing `index.js`:", e));

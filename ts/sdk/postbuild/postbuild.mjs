import { appendFileSync, readFileSync } from "fs";

// Paths
const PKG_DIR = `${import.meta.dirname}/../pkg`;
const INDEX_JS_PATH = `${PKG_DIR}/index.js`;
const INDEX_D_TS_PATH = `${PKG_DIR}/index.d.ts`;
const INDEX_BG_WASM_PATH = `${PKG_DIR}/index_bg.wasm`;

const WASM_B64_CONST_NAME = "WASM_BIN_B64";
const INIT_SYNC_EMBED_FN_NAME = "initSyncEmbed";

const CONST_D_TS_APPENDS = `
/**
 * Instantiates this \`module\` using the embedded
 * {@link ${WASM_B64_CONST_NAME}}. This is what works for nodejs envs.
 *
 * @returns {InitOutput}
 */
export function ${INIT_SYNC_EMBED_FN_NAME}(): InitOutput;

/**
 * Embedded base64-encoded wasm binary bytes
 */
export const ${WASM_B64_CONST_NAME}: string;
`;

const CONST_INDEX_JS_APPENDS = `
function ${INIT_SYNC_EMBED_FN_NAME}() {
  if (wasm !== undefined) return wasm;
  initSync({ module: Uint8Array.from(atob(${WASM_B64_CONST_NAME}), c => c.charCodeAt(0)) });
}

export { ${INIT_SYNC_EMBED_FN_NAME} };
`;

function indexJsWasmEmbedAppend() {
  const bytes = readFileSync(INDEX_BG_WASM_PATH);
  const b64 = bytes.toString("base64");
  return `
export const ${WASM_B64_CONST_NAME} = "${b64}";
`;
}

function main() {
  appendFileSync(INDEX_D_TS_PATH, CONST_D_TS_APPENDS);
  appendFileSync(INDEX_JS_PATH, CONST_INDEX_JS_APPENDS);
  appendFileSync(INDEX_JS_PATH, indexJsWasmEmbedAppend());
}

main();

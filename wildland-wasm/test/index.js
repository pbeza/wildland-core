import init, { get_version } from "../pkg/wildland_wasm.js";

const runWasm = async () => {
  await init();
  const version = get_version();
  document.body.textContent = `Wildland's Admin Manager version is: ${version}`;
};
runWasm();

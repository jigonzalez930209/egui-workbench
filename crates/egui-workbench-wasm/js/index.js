// ESM wrapper for the wasm host. The generated wasm-bindgen module lives in `../pkg`.
import init, { WebWorkbench } from "../pkg/egui_workbench_wasm.js";

let moduleReady;

async function load(wasmUrl) {
  if (!moduleReady) {
    moduleReady = init(wasmUrl);
  }
  return moduleReady;
}

function specText(spec) {
  return typeof spec === "string" ? spec : JSON.stringify(spec);
}

/**
 * Create a workbench on a canvas.
 * @param {{ spec: object | string, canvas: HTMLCanvasElement, wasmUrl?: string }} options
 */
export async function createWorkbench({ spec, canvas, wasmUrl }) {
  await load(wasmUrl);
  const native = new WebWorkbench(specText(spec));
  await native.start(canvas);
  return {
    native,
    applyPatch(patch) {
      native.apply_patch(JSON.stringify(patch));
    },
    setSpec(next) {
      native.set_spec(specText(next));
    },
    spec() {
      return JSON.parse(native.spec_json());
    },
    pollEvents() {
      return JSON.parse(native.poll_events_json());
    },
    setTheme(id) {
      native.set_theme(id);
    },
    schema() {
      return JSON.parse(WebWorkbench.schema());
    },
    destroy() {
      native.destroy();
    },
  };
}

function encode(message) {
  const json = new TextEncoder().encode(JSON.stringify(message));
  const bytes = new Uint8Array(4 + json.length);
  new DataView(bytes.buffer).setUint32(0, json.length);
  bytes.set(json, 4);
  return bytes.buffer;
}

function decode(buffer) {
  const bytes = new Uint8Array(buffer);
  const length = new DataView(buffer).getUint32(0);
  return JSON.parse(new TextDecoder().decode(bytes.subarray(4, 4 + length)));
}

/**
 * Render a host running on a server. The browser is the renderer; commands and patches
 * travel as the length-prefixed 0.11 messages inside WebSocket binary frames.
 * @param {{ url: string, canvas: HTMLCanvasElement, spec: object | string, wasmUrl?: string, onEvent?: (event: object) => void }} options
 */
export async function connectRemote({ url, canvas, spec, wasmUrl, onEvent }) {
  const workbench = await createWorkbench({ spec, canvas, wasmUrl });
  const socket = new WebSocket(url);
  socket.binaryType = "arraybuffer";
  let nextId = 0;
  const send = (request) => socket.send(encode({ type: "request", id: ++nextId, request }));

  socket.addEventListener("open", () => send({ op: "get_spec" }));
  socket.addEventListener("message", (message) => {
    const decoded = decode(message.data);
    if (decoded.type === "response" && decoded.response.result === "spec") {
      workbench.setSpec(decoded.response.spec);
    } else if (decoded.type === "event") {
      onEvent?.(decoded.event);
    }
  });
  const timer = setInterval(() => {
    for (const event of workbench.pollEvents()) {
      if (event.event === "command") {
        send({ op: "run_command", id: event.id });
      }
      onEvent?.(event);
    }
  }, 100);

  return {
    workbench,
    applyPatch(patch) {
      send({ op: "apply_patch", patch });
    },
    sendCommand(id) {
      send({ op: "run_command", id });
    },
    close() {
      clearInterval(timer);
      socket.close();
      workbench.destroy();
    },
  };
}

export { WebWorkbench };
export default createWorkbench;

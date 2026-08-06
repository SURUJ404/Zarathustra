// Astra frontend WASM loader (dependency-free ABI).
// Exposes an async default init and a `compile(source)` -> string export.
// The playground prefers this module when present and falls back to the
// `/api/*` backend otherwise.

let wasm = null;
let memory = null;
const enc = new TextEncoder();
const dec = new TextDecoder();

async function instantiate() {
    const url = new URL('./astra_frontend_bg.wasm', import.meta.url);
    let module;
    try {
        module = await WebAssembly.instantiateStreaming(fetch(url), {});
    } catch {
        const bytes = await (await fetch(url)).arrayBuffer();
        module = await WebAssembly.instantiate(bytes, {});
    }
    wasm = module.instance.exports;
    memory = wasm.memory;
    return wasm;
}

async function init() {
    if (wasm) return wasm;
    return instantiate();
}

function compile(source) {
    if (!wasm) {
        throw new Error('WASM not initialized: call default() before compile()');
    }
    const bytes = enc.encode(source);
    const inputPtr = bytes.length ? wasm.astra_alloc(bytes.length) : 0;
    if (bytes.length) {
        new Uint8Array(memory.buffer, inputPtr, bytes.length).set(bytes);
    }

    const slots = wasm.astra_alloc(8);
    const dv = new DataView(memory.buffer);
    const status = wasm.astra_compile(inputPtr, bytes.length, slots, slots + 4);
    const resultPtr = dv.getUint32(slots, true);
    const resultLen = dv.getUint32(slots + 4, true);
    const out = resultLen
        ? dec.decode(new Uint8Array(memory.buffer, resultPtr, resultLen))
        : '';

    if (resultPtr) wasm.astra_free(resultPtr, resultLen);
    wasm.astra_free(slots, 8);
    if (inputPtr) wasm.astra_free(inputPtr, bytes.length);

    if (status !== 0) throw new Error(out);
    return out;
}

export default init;
export { compile };

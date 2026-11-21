/**
 * Live demos start one after another: each iframe compiles the same wasm module, so the
 * queue lets the first demo on screen appear as fast as possible and keeps the browser
 * from compiling several copies at once. Later demos reuse the cached download.
 */
type Start = () => Promise<void>

const queue: Start[] = []
let busy = false

/** Run `start` when the demos queued before it are ready (or failed). */
export function enqueue(start: Start) {
  queue.push(start)
  void pump()
}

async function pump() {
  if (busy) return
  busy = true
  while (queue.length > 0) {
    const next = queue.shift()
    if (next) await next().catch(() => undefined)
  }
  busy = false
}

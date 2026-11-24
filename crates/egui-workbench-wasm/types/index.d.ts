import type { HostSpec } from "./schema.js";

export type Patch =
  | { op: "set"; path: string; value: unknown }
  | { op: "replace"; path: string; value: unknown }
  | { op: "remove"; path: string }
  | { op: "append"; path: string; values: unknown[] }
  | { op: "batch"; patches: Patch[] };

export interface Workbench {
  applyPatch(patch: Patch): void;
  setSpec(spec: HostSpec | string): void;
  spec(): HostSpec;
  pollEvents(): Array<Record<string, unknown>>;
  setTheme(id: string): void;
  schema(): Record<string, unknown>;
  destroy(): void;
}

export interface WorkbenchOptions {
  spec: HostSpec | string;
  canvas: HTMLCanvasElement;
  wasmUrl?: string;
}

export function createWorkbench(options: WorkbenchOptions): Promise<Workbench>;

export interface RemoteOptions extends WorkbenchOptions {
  url: string;
  onEvent?: (event: Record<string, unknown>) => void;
}

export interface RemoteWorkbench {
  workbench: Workbench;
  applyPatch(patch: Patch): void;
  sendCommand(id: string): void;
  close(): void;
}

export function connectRemote(options: RemoteOptions): Promise<RemoteWorkbench>;

export * from "./schema.js";

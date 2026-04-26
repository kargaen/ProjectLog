import { invoke } from "@tauri-apps/api/core";

type Level = "debug" | "info" | "warn" | "error";

function serialize(data: unknown): string | undefined {
  if (data === undefined) return undefined;
  if (data instanceof Error) return data.stack || data.message;
  if (typeof data === "string") return data;
  try {
    return JSON.stringify(data);
  } catch {
    return String(data);
  }
}

function send(level: Level, module: string, message: string, data?: unknown) {
  const payload = serialize(data);
  const prefix = `[${module}] ${message}`;
  const fn =
    level === "error" ? console.error :
    level === "warn" ? console.warn :
    level === "debug" ? console.debug :
    console.info;

  if (payload) fn(prefix, payload);
  else fn(prefix);

  invoke("log_from_frontend", {
    level,
    module,
    message,
    data: payload,
  }).catch(() => {});
}

export function createLogger(module: string) {
  return {
    debug: (message: string, data?: unknown) => send("debug", module, message, data),
    info: (message: string, data?: unknown) => send("info", module, message, data),
    warn: (message: string, data?: unknown) => send("warn", module, message, data),
    error: (message: string, data?: unknown) => send("error", module, message, data),
  };
}

export const logger = createLogger("app");

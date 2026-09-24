// Appels aux commandes natives. Aucune donnée n'est stockée côté interface : chaque succès
// affiché correspond à une réponse positive du cœur Rust.
import { invoke } from "@tauri-apps/api/core";

export interface AppError { kind: string; message: string }

export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauri()) {
    throw { kind: "no_app", message: "Cette interface doit être ouverte dans l'application CMME (aucune donnée n'est enregistrée dans un navigateur)." } as AppError;
  }
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    if (e && typeof e === "object" && "message" in e) throw e as AppError;
    throw { kind: "unknown", message: String(e) } as AppError;
  }
}

export function errMessage(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as AppError).message);
  return String(e);
}
export function errKind(e: unknown): string {
  if (e && typeof e === "object" && "kind" in e) return String((e as AppError).kind);
  return "unknown";
}

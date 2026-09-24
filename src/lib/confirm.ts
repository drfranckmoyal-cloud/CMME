// Confirmation par la boîte de dialogue native de macOS (via le greffon officiel Tauri).
import { ask } from "@tauri-apps/plugin-dialog";

export async function confirmAsk(message: string, okLabel = "Continuer"): Promise<boolean> {
  try {
    return await ask(message, { title: "CMME", kind: "warning", okLabel, cancelLabel: "Annuler" });
  } catch {
    return false;
  }
}

// Native file dialogs (Tauri plugin-dialog). All return absolute paths or
// null when the user cancels.

import { open, save } from "@tauri-apps/plugin-dialog";

export async function pickProjectFile(): Promise<string | null> {
  const sel = await open({
    title: "Open PlotVale project",
    filters: [{ name: "PlotVale projects", extensions: ["plotvale"] }],
  });
  return typeof sel === "string" ? sel : null;
}

export async function pickProjectFolder(): Promise<string | null> {
  const sel = await open({ title: "Open project folder", directory: true });
  return typeof sel === "string" ? sel : null;
}

export async function pickSaveBundle(suggestedName: string): Promise<string | null> {
  return await save({
    title: "Save PlotVale project",
    defaultPath: suggestedName.endsWith(".plotvale")
      ? suggestedName
      : `${suggestedName}.plotvale`,
    filters: [{ name: "PlotVale projects", extensions: ["plotvale"] }],
  });
}

export async function pickSaveFolder(): Promise<string | null> {
  const sel = await open({ title: "Save project folder", directory: true });
  return typeof sel === "string" ? sel : null;
}

export async function pickExportFile(
  suggestedName: string,
  displayName: string,
  extension: string,
): Promise<string | null> {
  const base = suggestedName.replace(/[\\/:*?"<>|]/g, "_").trim() || "manuscript";
  return await save({
    title: "Export as",
    defaultPath: `${base}.${extension}`,
    filters: [{ name: displayName, extensions: [extension] }],
  });
}

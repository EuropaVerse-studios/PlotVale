// Native file dialogs (Tauri plugin-dialog). All return absolute paths or
// null when the user cancels.
// Titles and filter names come from the callers (already translated with
// t()): nothing user-visible is hardcoded here.

import { open, save } from "@tauri-apps/plugin-dialog";

export async function pickProjectFile(
  title: string,
  filterName: string,
): Promise<string | null> {
  const sel = await open({
    title,
    filters: [{ name: filterName, extensions: ["plotvale"] }],
  });
  return typeof sel === "string" ? sel : null;
}

export async function pickProjectFolder(title: string): Promise<string | null> {
  const sel = await open({ title, directory: true });
  return typeof sel === "string" ? sel : null;
}

export async function pickSaveBundle(
  suggestedName: string,
  title: string,
  filterName: string,
): Promise<string | null> {
  return await save({
    title,
    defaultPath: suggestedName.endsWith(".plotvale")
      ? suggestedName
      : `${suggestedName}.plotvale`,
    filters: [{ name: filterName, extensions: ["plotvale"] }],
  });
}

export async function pickSaveFolder(title: string): Promise<string | null> {
  const sel = await open({ title, directory: true });
  return typeof sel === "string" ? sel : null;
}

export async function pickExportFile(
  suggestedName: string,
  displayName: string,
  extension: string,
  dialogTitle: string,
  fallbackName: string,
): Promise<string | null> {
  const base = suggestedName.replace(/[\\/:*?"<>|]/g, "_").trim() || fallbackName;
  return await save({
    title: dialogTitle,
    defaultPath: `${base}.${extension}`,
    filters: [{ name: displayName, extensions: [extension] }],
  });
}

// Typed wrappers over every Tauri command (see src-tauri/src/commands.rs).
// All errors surface as Error with the backend's message.

import { invoke } from "@tauri-apps/api/core";
import type {
  CharacterSheet,
  CollectionDef,
  CompileProfile,
  HitDto,
  InitialOpen,
  LocationSheet,
  NodeDto,
  NodeMeta,
  ProfilesFile,
  ProjectDto,
  ProjectMeta,
  ProjectStatus,
  SaveInfo,
  StatsDto,
  Targets,
  SnapshotMeta,
  WordCounts,
} from "./types";

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new Error(typeof e === "string" ? e : `${e}`);
  }
}

export const api = {
  initialProject: () => call<InitialOpen | null>("initial_project"),
  newProject: (title: string, author: string) =>
    call<ProjectDto>("new_project", { title, author }),
  openProject: (path: string) => call<ProjectDto>("open_project", { path }),
  saveProject: () => call<SaveInfo>("save_project"),
  saveProjectAs: (path: string, folderMode: boolean) =>
    call<SaveInfo>("save_project_as", { path, folderMode }),
  closeProject: () => call<void>("close_project"),
  projectStatus: () => call<ProjectStatus>("project_status"),

  createNode: (parent: string, kind: string, title: string) =>
    call<NodeDto>("create_node", { parent, kind, title }),
  renameNode: (id: string, title: string) => call<void>("rename_node", { id, title }),
  moveNode: (id: string, newParent: string, pos: number) =>
    call<void>("move_node", { id, newParent, pos }),
  reorderNode: (id: string, up: boolean) => call<boolean>("reorder_node", { id, up }),
  indentNode: (id: string) => call<boolean>("indent_node", { id }),
  outdentNode: (id: string) => call<boolean>("outdent_node", { id }),
  trashNode: (id: string) => call<void>("trash_node", { id }),
  restoreNode: (id: string) => call<void>("restore_node", { id }),
  emptyTrash: () => call<string[]>("empty_trash"),
  deleteNode: (id: string) => call<string[]>("delete_node", { id }),
  getTree: () => call<NodeDto[]>("get_tree"),
  getNode: (id: string) => call<NodeDto>("get_node", { id }),

  getDocText: (id: string) => call<string>("get_doc_text", { id }),
  setDocText: (id: string, text: string) => call<void>("set_doc_text", { id, text }),

  updateNodeMeta: (id: string, meta: NodeMeta) =>
    call<void>("update_node_meta", { id, meta }),
  getProjectMeta: () => call<ProjectMeta>("get_project_meta"),
  setProjectMeta: (meta: ProjectMeta) => call<void>("set_project_meta", { meta }),
  upsertCharacter: (sheet: CharacterSheet) =>
    call<void>("upsert_character", { sheet }),
  upsertLocation: (sheet: LocationSheet) => call<void>("upsert_location", { sheet }),

  getTargets: () => call<Targets>("get_targets"),
  setTargets: (targets: Targets) => call<void>("set_targets", { targets }),

  listSnapshots: (docId: string) => call<SnapshotMeta[]>("list_snapshots", { docId }),
  createSnapshot: (docId: string, label: string) =>
    call<SnapshotMeta>("create_snapshot", { docId, label }),
  getSnapshotText: (snapId: string) =>
    call<string>("get_snapshot_text", { snapId }),
  restoreSnapshot: (docId: string, snapId: string) =>
    call<void>("restore_snapshot", { docId, snapId }),
  deleteSnapshot: (snapId: string) => call<void>("delete_snapshot", { snapId }),

  searchProject: (pattern: string, regex: boolean, caseSensitive: boolean) =>
    call<HitDto[]>("search_project", { pattern, regex, caseSensitive }),

  projectStats: () => call<StatsDto>("project_stats"),
  docWords: (id: string) => call<WordCounts>("doc_words", { id }),
  totalWords: () => call<number>("total_words"),
  subtreeWords: (id: string) => call<number>("subtree_words", { id }),

  listProfiles: () => call<ProfilesFile>("list_profiles"),
  setActiveProfile: (name: string) => call<void>("set_active_profile", { name }),
  updateProfile: (profile: CompileProfile) =>
    call<void>("update_profile", { profile }),
  compilePreview: (profileName: string | null, keepNotes: boolean | null, selection: string[]) =>
    call<string>("compile_preview", { profileName, keepNotes, selection }),
  exportManuscript: (
    format: string,
    path: string,
    profileName: string | null,
    keepNotes: boolean | null,
    selection: string[],
  ) => call<void>("export_manuscript", { format, path, profileName, keepNotes, selection }),

  listCollections: () => call<CollectionDef[]>("list_collections"),
  addCollection: (collection: CollectionDef) =>
    call<void>("add_collection", { collection }),
  removeCollection: (name: string) => call<void>("remove_collection", { name }),
};

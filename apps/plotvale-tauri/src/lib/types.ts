// Mirrors of the Rust DTOs in src-tauri/src/state.rs + core Serialize types.
// Node ids travel as ULID strings; kinds as snake_case (serde rename_all).

export type NodeKind =
  | "draft_root"
  | "folder"
  | "scene"
  | "note"
  | "research_folder"
  | "research_item"
  | "character_sheet"
  | "location_sheet"
  | "trash";

export type DocStatus = "todo" | "draft" | "revised" | "final";

export interface NodeMeta {
  status: DocStatus;
  label: string;
  keywords: string[];
  synopsis: string;
  include_in_export: boolean;
  color: string | null;
  custom: Record<string, string>;
  trashed_from: string | null;
}

export interface NodeDto {
  id: string;
  parent: string | null;
  children: string[];
  kind: NodeKind;
  title: string;
  meta: NodeMeta;
}

export interface Manifest {
  schema_version: number;
  app_version: string;
  title: string;
  author: string;
  language: string;
  created_at: string;
  updated_at: string;
}

export interface CollectionDef {
  name: string;
  pattern: string;
  regex: boolean;
  case_sensitive: boolean;
}

export interface CustomField {
  name: string;
  hint: string;
}

export interface ProjectMeta {
  keywords: string[];
  collections: CollectionDef[];
  custom_fields: CustomField[];
  notes: string;
}

export interface CharacterSheet {
  node_id: string;
  name: string;
  role: string;
  description: string;
  goal: string;
  conflict: string;
  questionnaire: Record<string, string>;
}

export interface LocationSheet {
  node_id: string;
  name: string;
  description: string;
  sensory: string;
  notes: string;
}

export interface Targets {
  project_goal_words: number | null;
  daily_goal_words: number | null;
  words_today: number;
  words_today_date: string | null;
}

export interface SnapshotMeta {
  id: string;
  doc_id: string;
  created_at: string;
  label: string;
}

export interface CompileProfile {
  name: string;
  show_scene_titles: boolean;
  scene_separator: string;
  page_break_before_chapters: boolean;
  keep_notes: boolean;
  dedication: string;
}

export interface ProfilesFile {
  profiles: CompileProfile[];
  active: string;
}

export interface ProjectDto {
  manifest: Manifest;
  nodes: NodeDto[];
  draft_root: string;
  research_root: string;
  trash: string;
  docs: Record<string, string>;
  meta: ProjectMeta;
  characters: CharacterSheet[];
  locations: LocationSheet[];
  targets: Targets;
  snapshots: SnapshotMeta[];
  profiles: ProfilesFile;
}

export interface ProjectStatus {
  has_project: boolean;
  path: string | null;
  folder_mode: boolean;
  dirty: boolean;
  title: string;
}

export interface SaveInfo {
  total_words: number;
  words_today: number;
}

export interface HitDto {
  doc_id: string;
  title: string;
  line_no: number;
  col: number;
  excerpt: string;
}

export interface ChapterWords {
  id: string;
  title: string;
  words: number;
}

export interface WordCounts {
  words: number;
  characters: number;
  characters_no_spaces: number;
  paragraphs: number;
}

export interface StatsDto {
  words: number;
  characters_no_spaces: number;
  paragraphs: number;
  docs_with_text: number;
  per_chapter: ChapterWords[];
  total_counts: WordCounts;
}

/** Nodes that can hold editable text (mirrors NodeKind::can_have_text). */
export function canHaveText(kind: NodeKind): boolean {
  return (
    kind === "scene" ||
    kind === "note" ||
    kind === "research_item" ||
    kind === "character_sheet" ||
    kind === "location_sheet"
  );
}

/** System roots: neither movable nor deletable. */
export function isSystemKind(kind: NodeKind): boolean {
  return kind === "draft_root" || kind === "research_folder" || kind === "trash";
}

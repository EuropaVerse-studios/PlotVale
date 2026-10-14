// Project store: mirrors the backend DTO for rendering, syncs deltas back.
// Texts edit locally (instant, even on 150k-word scenes) and flush to Rust
// debounced; tree mutations refetch the (small) tree for consistency.

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { ReactNode } from "react";
import { api } from "./api";
import { countWords } from "./count";
import type {
  CharacterSheet,
  LocationSheet,
  NodeDto,
  NodeMeta,
  ProjectDto,
  Targets,
} from "./types";
import { canHaveText } from "./types";

export type CenterTab = "editor" | "corkboard" | "search";

/** Idle ms before autosave kicks in (classic UI used 3 s). */
const AUTOSAVE_MS = 3000;
/** Keystroke debounce before flushing text to the backend. */
const SYNC_TEXT_MS = 600;

interface Store {
  project: ProjectDto | null;
  nodes: Map<string, NodeDto>;
  draftRoot: string;
  researchRoot: string;
  trashId: string;
  selection: string | null;
  openDoc: string | null;
  texts: Record<string, string>;
  /** Bumped when a text changes outside the editor (restore, open). */
  textRev: Record<string, number>;
  dirty: boolean;
  hasPath: boolean;
  path: string | null;
  folderMode: boolean;
  status: string;
  expert: boolean;
  scrivenings: boolean;
  collapsed: Set<string>;
  fontSize: number;
  centerTab: CenterTab;
  totalWords: number;
  openDocWords: number;

  setSelection: (id: string | null) => void;
  setOpenDoc: (id: string | null) => void;
  setCenterTab: (tab: CenterTab) => void;
  setExpert: (v: boolean) => void;
  setScrivenings: (v: boolean) => void;
  setFontSize: (v: number) => void;
  toggleCollapsed: (id: string) => void;
  setStatus: (msg: string) => void;

  doNew: (title: string, author: string) => Promise<void>;
  doOpen: (path: string) => Promise<void>;
  doSave: () => Promise<boolean>;
  doSaveAs: (path: string, folderMode: boolean) => Promise<boolean>;
  doClose: () => Promise<void>;
  setText: (id: string, text: string) => void;
  flushText: (id: string) => Promise<void>;

  createNode: (parent: string, kind: string, title: string) => Promise<NodeDto>;
  renameNode: (id: string, title: string) => Promise<void>;
  moveNode: (id: string, newParent: string, pos: number) => Promise<void>;
  reorderNode: (id: string, up: boolean) => Promise<void>;
  indentNode: (id: string) => Promise<void>;
  outdentNode: (id: string) => Promise<void>;
  trashNode: (id: string) => Promise<void>;
  restoreNode: (id: string) => Promise<void>;
  emptyTrash: () => Promise<void>;
  deleteNode: (id: string) => Promise<void>;
  updateNodeMeta: (id: string, meta: NodeMeta) => Promise<void>;
  upsertCharacter: (sheet: CharacterSheet) => Promise<void>;
  upsertLocation: (sheet: LocationSheet) => Promise<void>;
  setTargets: (targets: Targets) => Promise<void>;
  /** Overwrites one open text from the backend (snapshot restore). */
  adoptBackendText: (id: string) => Promise<void>;
}

const Ctx = createContext<Store | null>(null);

function fail(setStatus: (m: string) => void, prefix: string, e: unknown): void {
  setStatus(`${prefix}: ${e instanceof Error ? e.message : `${e}`}`);
}

export function StoreProvider({ children }: { children: ReactNode }) {
  const [project, setProject] = useState<ProjectDto | null>(null);
  const [selection, setSelection] = useState<string | null>(null);
  const [openDoc, setOpenDoc] = useState<string | null>(null);
  const [texts, setTexts] = useState<Record<string, string>>({});
  const [textRev, setTextRev] = useState<Record<string, number>>({});
  const [dirty, setDirty] = useState(false);
  const [hasPath, setHasPath] = useState(false);
  const [path, setPath] = useState<string | null>(null);
  const [folderMode, setFolderMode] = useState(false);
  const [status, setStatus] = useState("");
  const [expert, setExpert] = useState(false);
  const [scrivenings, setScrivenings] = useState(false);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [fontSize, setFontSize] = useState(17);
  const [centerTab, setCenterTab] = useState<CenterTab>("editor");

  const lastEdit = useRef(0);
  const syncTimers = useRef<Record<string, number>>({});
  const stateRef = useRef({ dirty, hasPath });
  stateRef.current = { dirty, hasPath };

  const nodes = useMemo(() => {
    const map = new Map<string, NodeDto>();
    for (const n of project?.nodes ?? []) map.set(n.id, n);
    return map;
  }, [project]);

  const draftRoot = project?.draft_root ?? "";
  const researchRoot = project?.research_root ?? "";
  const trashId = project?.trash ?? "";

  const isInTrash = useCallback(
    (id: string): boolean => {
      let cur: string | null | undefined = id;
      while (cur) {
        if (cur === trashId) return true;
        cur = nodes.get(cur)?.parent;
      }
      return false;
    },
    [nodes, trashId],
  );

  const totalWords = useMemo(() => {
    if (!project) return 0;
    let total = 0;
    for (const [id, text] of Object.entries(texts)) {
      if (!isInTrash(id)) total += countWords(text).words;
    }
    return total;
  }, [project, texts, isInTrash]);

  const openDocWords = useMemo(() => {
    if (!openDoc) return 0;
    return countWords(texts[openDoc] ?? "").words;
  }, [openDoc, texts]);

  const install = useCallback((dto: ProjectDto, savePath: string | null, folder: boolean) => {
    setProject(dto);
    setTexts({ ...dto.docs });
    setTextRev({});
    setSelection(dto.draft_root);
    // First manuscript scene in tree order comes to the front, like the classic UI.
    const byId = new Map(dto.nodes.map((n) => [n.id, n] as const));
    const walk: string[] = [];
    const visit = (id: string): void => {
      const node = byId.get(id);
      if (!node) return;
      walk.push(id);
      for (const child of node.children) visit(child);
    };
    visit(dto.draft_root);
    const firstManuscript = walk.find((id) => canHaveText(byId.get(id)?.kind ?? "folder"));
    setOpenDoc(firstManuscript ?? null);
    setCenterTab("editor");
    setCollapsed(new Set());
    setDirty(savePath === null);
    setHasPath(savePath !== null);
    setPath(savePath);
    setFolderMode(folder);
    setStatus("");
  }, []);

  const doNew = useCallback(
    async (title: string, author: string) => {
      try {
        install(await api.newProject(title, author), null, false);
      } catch (e) {
        fail(setStatus, "New project failed", e);
      }
    },
    [install],
  );

  const doOpen = useCallback(
    async (openPath: string) => {
      try {
        const folder = await isDirectoryHint(openPath);
        install(await api.openProject(openPath), openPath, folder);
      } catch (e) {
        fail(setStatus, "Open failed", e);
      }
    },
    [install],
  );

  const doSave = useCallback(async (): Promise<boolean> => {
    // Flush pending text syncs first: saves never lose keystrokes.
    for (const id of Object.keys(syncTimers.current)) {
      window.clearTimeout(syncTimers.current[id]);
    }
    syncTimers.current = {};
    try {
      const entries = await snapshotTexts();
      for (const [id, text] of entries) await api.setDocText(id, text);
      const info = await api.saveProject();
      void info;
      setDirty(false);
      return true;
    } catch (e) {
      fail(setStatus, "Save failed", e);
      return false;
    }
  }, []);

  const textsRef = useRef(texts);
  textsRef.current = texts;
  async function snapshotTexts(): Promise<[string, string][]> {
    return Object.entries(textsRef.current);
  }

  const doSaveAs = useCallback(
    async (savePath: string, folder: boolean): Promise<boolean> => {
      for (const id of Object.keys(syncTimers.current)) {
        window.clearTimeout(syncTimers.current[id]);
      }
      syncTimers.current = {};
      try {
        for (const [id, text] of Object.entries(textsRef.current)) {
          await api.setDocText(id, text);
        }
        await api.saveProjectAs(savePath, folder);
        setDirty(false);
        setHasPath(true);
        setPath(savePath);
        setFolderMode(folder);
        return true;
      } catch (e) {
        fail(setStatus, "Save failed", e);
        return false;
      }
    },
    [],
  );

  const doClose = useCallback(async () => {
    try {
      await api.closeProject();
    } catch (e) {
      fail(setStatus, "Close failed", e);
      return;
    }
    setProject(null);
    setTexts({});
    setTextRev({});
    setSelection(null);
    setOpenDoc(null);
    setDirty(false);
    setHasPath(false);
    setPath(null);
  }, []);

  const setText = useCallback((id: string, text: string) => {
    setTexts((prev) => (prev[id] === text ? prev : { ...prev, [id]: text }));
    setDirty(true);
    lastEdit.current = Date.now();
    window.clearTimeout(syncTimers.current[id]);
    syncTimers.current[id] = window.setTimeout(() => {
      delete syncTimers.current[id];
      api.setDocText(id, text).catch(() => {});
    }, SYNC_TEXT_MS);
  }, []);

  const flushText = useCallback(async (id: string) => {
    window.clearTimeout(syncTimers.current[id]);
    delete syncTimers.current[id];
    const text = textsRef.current[id];
    if (text === undefined) return;
    try {
      await api.setDocText(id, text);
    } catch (e) {
      fail(setStatus, "Sync failed", e);
    }
  }, []);

  // Autosave after 3 s idle (only when a save location exists).
  useEffect(() => {
    const timer = window.setInterval(() => {
      const { dirty: d, hasPath: hp } = stateRef.current;
      if (d && hp && Date.now() - lastEdit.current > AUTOSAVE_MS) {
        void (async () => {
          for (const [id, text] of Object.entries(textsRef.current)) {
            try {
              await api.setDocText(id, text);
            } catch {
              return;
            }
          }
          try {
            await api.saveProject();
            setDirty(false);
          } catch {
            /* status keeps the error visible on manual save */
          }
        })();
      }
    }, 1000);
    return () => window.clearInterval(timer);
  }, []);

  const refreshTree = useCallback(async () => {
    const fresh = await api.getTree();
    setProject((prev) => (prev ? { ...prev, nodes: fresh } : prev));
    return fresh;
  }, []);

  const reconcileSelection = useCallback(
    (fresh: NodeDto[]) => {
      const ids = new Set(fresh.map((n) => n.id));
      setSelection((sel) => (sel && ids.has(sel) ? sel : (fresh[0]?.id ?? null)));
      setOpenDoc((open) => (open && ids.has(open) ? open : null));
      setTexts((prev) => {
        const next: Record<string, string> = {};
        for (const [id, text] of Object.entries(prev)) {
          if (ids.has(id)) next[id] = text;
        }
        return next;
      });
    },
    [],
  );

  const mutate = useCallback(
    async (op: () => Promise<unknown>, errPrefix: string) => {
      try {
        await op();
        reconcileSelection(await refreshTree());
      } catch (e) {
        fail(setStatus, errPrefix, e);
        try {
          reconcileSelection(await refreshTree());
        } catch {
          /* keep the local mirror on total backend failure */
        }
      }
    },
    [reconcileSelection, refreshTree],
  );

  const createNode = useCallback(
    async (parent: string, kind: string, title: string): Promise<NodeDto> => {
      const dto = await api.createNode(parent, kind, title);
      if (canHaveText(dto.kind as NodeDto["kind"])) {
        setTexts((prev) => ({ ...prev, [dto.id]: "" }));
      }
      setSelection(dto.id);
      await refreshTree();
      return dto;
    },
    [refreshTree],
  );

  const renameNode = useCallback(
    async (id: string, title: string) => {
      await mutate(
        async () => {
          await api.renameNode(id, title);
          setProject((prev) =>
            prev
              ? {
                  ...prev,
                  nodes: prev.nodes.map((n) => (n.id === id ? { ...n, title } : n)),
                }
              : prev,
          );
        },
        "Rename failed",
      );
    },
    [mutate],
  );

  const moveNode = useCallback(
    async (id: string, newParent: string, pos: number) => {
      await mutate(() => api.moveNode(id, newParent, pos), "Move failed");
    },
    [mutate],
  );

  const reorderNode = useCallback(
    async (id: string, up: boolean) => {
      await mutate(() => api.reorderNode(id, up), "Reorder failed");
    },
    [mutate],
  );

  const indentNode = useCallback(
    async (id: string) => {
      await mutate(() => api.indentNode(id), "Indent failed");
    },
    [mutate],
  );

  const outdentNode = useCallback(
    async (id: string) => {
      await mutate(() => api.outdentNode(id), "Outdent failed");
    },
    [mutate],
  );

  const trashNode = useCallback(
    async (id: string) => {
      await mutate(() => api.trashNode(id), "Trash failed");
    },
    [mutate],
  );

  const restoreNode = useCallback(
    async (id: string) => {
      await mutate(() => api.restoreNode(id), "Restore failed");
    },
    [mutate],
  );

  const emptyTrash = useCallback(async () => {
    await mutate(() => api.emptyTrash(), "Empty trash failed");
  }, [mutate]);

  const deleteNode = useCallback(
    async (id: string) => {
      await mutate(() => api.deleteNode(id), "Delete failed");
    },
    [mutate],
  );

  const updateNodeMeta = useCallback(
    async (id: string, meta: NodeMeta) => {
      try {
        await api.updateNodeMeta(id, meta);
        setProject((prev) =>
          prev
            ? { ...prev, nodes: prev.nodes.map((n) => (n.id === id ? { ...n, meta } : n)) }
            : prev,
        );
        setDirty(true);
      } catch (e) {
        fail(setStatus, "Inspector save failed", e);
      }
    },
    [],
  );

  const upsertCharacter = useCallback(async (sheet: CharacterSheet) => {
    try {
      await api.upsertCharacter(sheet);
      setProject((prev) =>
        prev
          ? {
              ...prev,
              characters: prev.characters.some((s) => s.node_id === sheet.node_id)
                ? prev.characters.map((s) => (s.node_id === sheet.node_id ? sheet : s))
                : [...prev.characters, sheet],
            }
          : prev,
      );
      setDirty(true);
    } catch (e) {
      fail(setStatus, "Sheet save failed", e);
    }
  }, []);

  const upsertLocation = useCallback(async (sheet: LocationSheet) => {
    try {
      await api.upsertLocation(sheet);
      setProject((prev) =>
        prev
          ? {
              ...prev,
              locations: prev.locations.some((s) => s.node_id === sheet.node_id)
                ? prev.locations.map((s) => (s.node_id === sheet.node_id ? sheet : s))
                : [...prev.locations, sheet],
            }
          : prev,
      );
      setDirty(true);
    } catch (e) {
      fail(setStatus, "Sheet save failed", e);
    }
  }, []);

  const setTargets = useCallback(async (targets: Targets) => {
    try {
      await api.setTargets(targets);
      setProject((prev) => (prev ? { ...prev, targets } : prev));
      setDirty(true);
    } catch (e) {
      fail(setStatus, "Goals save failed", e);
    }
  }, []);

  const adoptBackendText = useCallback(async (id: string) => {
    try {
      const text = await api.getDocText(id);
      setTexts((prev) => ({ ...prev, [id]: text }));
      setTextRev((prev) => ({ ...prev, [id]: (prev[id] ?? 0) + 1 }));
      setDirty(true);
    } catch (e) {
      fail(setStatus, "Refresh failed", e);
    }
  }, []);

  const toggleCollapsed = useCallback((id: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const value: Store = {
    project,
    nodes,
    draftRoot,
    researchRoot,
    trashId,
    selection,
    openDoc,
    texts,
    textRev,
    dirty,
    hasPath,
    path,
    folderMode,
    status,
    expert,
    scrivenings,
    collapsed,
    fontSize,
    centerTab,
    totalWords,
    openDocWords,
    setSelection,
    setOpenDoc,
    setCenterTab,
    setExpert,
    setScrivenings,
    setFontSize,
    toggleCollapsed,
    setStatus,
    doNew,
    doOpen,
    doSave,
    doSaveAs,
    doClose,
    setText,
    flushText,
    createNode,
    renameNode,
    moveNode,
    reorderNode,
    indentNode,
    outdentNode,
    trashNode,
    restoreNode,
    emptyTrash,
    deleteNode,
    updateNodeMeta,
    upsertCharacter,
    upsertLocation,
    setTargets,
    adoptBackendText,
  };

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useStore(): Store {
  const store = useContext(Ctx);
  if (!store) throw new Error("useStore outside StoreProvider");
  return store;
}

/** Heuristic: folders vs `.plotvale` bundles (the backend decides for real). */
async function isDirectoryHint(p: string): Promise<boolean> {
  return !p.toLowerCase().endsWith(".plotvale");
}

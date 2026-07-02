'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  AlertTriangle,
  Camera,
  Check,
  FileText,
  History as HistoryIcon,
  Layers,
  Network,
  Pin,
  PinOff,
  Plus,
  Search,
  Settings,
  Tag as TagIcon,
  Trash2,
  X,
} from 'lucide-react';
import { basesApi, foldersApi, notesApi, pickVaultDirectory, tagsApi, vaultsApi } from '@/lib/tauri-api';
import { GraphView } from '@/components/GraphView';
import { CreateBaseModal } from '@/components/CreateBaseModal';
import { NoteContextMenu } from '@/components/NoteContextMenu';
import { SearchModal } from '@/components/SearchModal';
import { SettingsModal } from '@/components/SettingsModal';
import { formatDate, formatRelativeTime } from '@/lib/format';
import type {
  Backlink,
  Base,
  BaseNoteAssignment,
  Folder,
  Link as WikiLink,
  Note,
  NoteSummary,
  NoteTagAssignment,
  NoteVersion,
  Tag,
  VaultInfo,
} from '@/lib/types';

export default function Page() {
  const [vaults, setVaults] = useState<VaultInfo[]>([]);
  const [activeVault, setActiveVault] = useState<VaultInfo | null>(null);
  const [loadingVaults, setLoadingVaults] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshVaults = useCallback(async () => {
    try {
      const list = await vaultsApi.list();
      setVaults(list);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoadingVaults(false);
    }
  }, []);

  useEffect(() => {
    refreshVaults();
  }, [refreshVaults]);

  if (loadingVaults) return <CenteredMessage>Loading…</CenteredMessage>;

  if (!activeVault) {
    return (
      <VaultGate
        vaults={vaults}
        error={error}
        onOpened={setActiveVault}
        onCreated={(v) => {
          setVaults((prev) => [...prev, v]);
          setActiveVault(v);
        }}
        onError={setError}
      />
    );
  }

  return <Workspace vault={activeVault} onCloseVault={() => setActiveVault(null)} />;
}

function CenteredMessage({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen items-center justify-center text-sm text-zinc-500">{children}</div>
  );
}

function VaultGate({
  vaults, error, onOpened, onCreated, onError,
}: {
  vaults: VaultInfo[];
  error: string | null;
  onOpened: (v: VaultInfo) => void;
  onCreated: (v: VaultInfo) => void;
  onError: (msg: string) => void;
}) {
  const [name, setName] = useState('');
  const [path, setPath] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmingId, setConfirmingId] = useState<string | null>(null);
  const [localVaults, setLocalVaults] = useState(vaults);

  useEffect(() => {
    setLocalVaults(vaults);
  }, [vaults]);

  const choosePath = async () => {
    try {
      const chosen = await pickVaultDirectory();
      if (chosen) setPath(chosen);
    } catch (e) {
      onError(String(e));
    }
  };

  const create = async () => {
    if (!name.trim() || !path) return;
    setBusy(true);
    try {
      onCreated(await vaultsApi.create(name.trim(), path));
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const open = async (vaultId: string) => {
    setBusy(true);
    try {
      onOpened(await vaultsApi.open(vaultId));
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const removeVault = async (vaultId: string, deleteFiles: boolean) => {
    setBusy(true);
    try {
      await vaultsApi.remove(vaultId, deleteFiles);
      setLocalVaults((prev) => prev.filter((v) => v.id !== vaultId));
      setConfirmingId(null);
    } catch (e) {
      onError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex h-screen items-center justify-center px-6">
      <div className="w-full max-w-md space-y-8">
        <div>
          <h1 className="text-2xl font-semibold text-zinc-100">Your vaults</h1>
          <p className="mt-1 text-sm text-zinc-500">
            A vault is a folder on your disk — each one is a self-contained note database.
          </p>
        </div>

        {error && (
          <p className="rounded-md bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p>
        )}

        {localVaults.length > 0 && (
          <ul className="space-y-2">
            {localVaults.map((v) => (
              <li key={v.id} className="rounded-lg border border-ink-700 bg-ink-900">
                {confirmingId === v.id ? (
                  <div className="space-y-2 p-4">
                    <div className="flex items-start gap-2 text-sm text-zinc-300">
                      <AlertTriangle size={15} className="mt-0.5 shrink-0 text-amber-400" />
                      <span>Remove &ldquo;{v.name}&rdquo; from this list, or delete its files permanently?</span>
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                      <button
                        onClick={() => removeVault(v.id, false)}
                        disabled={busy}
                        className="rounded-md border border-ink-700 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-ink-800 disabled:opacity-50"
                      >
                        Remove from list (keep files)
                      </button>
                      <button
                        onClick={() => removeVault(v.id, true)}
                        disabled={busy}
                        className="rounded-md bg-red-900/60 px-2.5 py-1.5 text-xs font-medium text-red-200 hover:bg-red-900 disabled:opacity-50"
                      >
                        Delete permanently
                      </button>
                      <button
                        onClick={() => setConfirmingId(null)}
                        disabled={busy}
                        className="rounded-md px-2.5 py-1.5 text-xs text-zinc-500 hover:text-zinc-300"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="flex items-center">
                    <button
                      onClick={() => open(v.id)}
                      disabled={busy}
                      className="flex min-w-0 flex-1 items-center justify-between px-4 py-3 text-left transition hover:bg-ink-800 disabled:opacity-50"
                    >
                      <span className="font-medium text-zinc-100">{v.name}</span>
                      <span className="truncate pl-4 text-xs text-zinc-500">{v.path}</span>
                    </button>
                    <button
                      onClick={() => setConfirmingId(v.id)}
                      title="Remove this vault"
                      className="shrink-0 px-3 py-3 text-zinc-600 hover:text-red-400"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}

        <div className="rounded-lg border border-dashed border-ink-600 p-4">
          <h2 className="mb-3 text-sm font-medium text-zinc-300">Create a new vault</h2>
          <div className="space-y-3">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') path ? create() : choosePath();
              }}
              placeholder="Vault name"
              className="w-full rounded-md border border-ink-700 bg-ink-900 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-accent focus:outline-none"
            />
            <button
              onClick={choosePath}
              className="w-full rounded-md border border-ink-700 bg-ink-900 px-3 py-2 text-left text-sm text-zinc-400 hover:border-accent/60"
            >
              {path ?? 'Choose folder…'}
            </button>
            <button
              onClick={create}
              disabled={busy || !name.trim() || !path}
              className="w-full rounded-md bg-accent px-3 py-2 text-sm font-medium text-white transition hover:bg-accent-dim disabled:cursor-not-allowed disabled:opacity-40"
            >
              {busy ? 'Creating…' : 'Create vault'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

type MainView = 'notes' | 'graph';

function Workspace({ vault, onCloseVault }: { vault: VaultInfo; onCloseVault: () => void }) {
  const [folders, setFolders] = useState<Folder[]>([]);
  const [notes, setNotes] = useState<NoteSummary[]>([]);
  const [allLinks, setAllLinks] = useState<WikiLink[]>([]);
  const [noteTagAssignments, setNoteTagAssignments] = useState<NoteTagAssignment[]>([]);
  const [allTags, setAllTags] = useState<Tag[]>([]);
  const [bases, setBases] = useState<Base[]>([]);
  const [baseNoteAssignments, setBaseNoteAssignments] = useState<BaseNoteAssignment[]>([]);

  const [activeNote, setActiveNote] = useState<Note | null>(null);
  const [backlinks, setBacklinks] = useState<Backlink[]>([]);
  const [outgoing, setOutgoing] = useState<WikiLink[]>([]);
  const [tags, setTags] = useState<Tag[]>([]);
  const [versions, setVersions] = useState<NoteVersion[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  const [draftContent, setDraftContent] = useState('');
  const [draftTitle, setDraftTitle] = useState('');
  const [newTagName, setNewTagName] = useState('');
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  const [snapshotJustSaved, setSnapshotJustSaved] = useState(false);

  const [view, setView] = useState<MainView>('notes');
  const [sidebarMenu, setSidebarMenu] = useState<{ note: NoteSummary; x: number; y: number } | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [showCreateBase, setShowCreateBase] = useState(false);
  const [sortMode, setSortMode] = useState<'recent' | 'alphabetical'>('recent');
  const [error, setError] = useState<string | null>(null);

  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const titleTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const snapshotFeedbackTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const activeNoteIdRef = useRef<string | null>(null);

  const refreshNotes = useCallback(async () => {
    try {
      const [noteList, folderList, linkList, tagAssignments, tagList, baseList, baseAssignments] = await Promise.all([
        notesApi.list(null, false),
        foldersApi.list(),
        notesApi.allLinks(),
        tagsApi.allNoteTags(),
        tagsApi.list(),
        basesApi.list(),
        basesApi.allBaseNotes(),
      ]);
      setNotes(noteList);
      setFolders(folderList);
      setAllLinks(linkList);
      setNoteTagAssignments(tagAssignments);
      setAllTags(tagList);
      setBases(baseList);
      setBaseNoteAssignments(baseAssignments);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refreshNotes();
  }, [refreshNotes]);

  const noteColors = useMemo(() => {
    const map: Record<string, string | undefined> = {};
    for (const a of noteTagAssignments) {
      if (a.color && !map[a.note_id]) map[a.note_id] = a.color;
    }
    for (const n of notes) {
      if (n.color) map[n.id] = n.color;
    }
    return map;
  }, [noteTagAssignments, notes]);

  const { pinnedNotes, otherNotes } = useMemo(() => {
    const sorter =
      sortMode === 'alphabetical'
        ? (a: NoteSummary, b: NoteSummary) => a.title.localeCompare(b.title)
        : (a: NoteSummary, b: NoteSummary) => b.updated_at.localeCompare(a.updated_at);
    return {
      pinnedNotes: notes.filter((n) => n.is_pinned).sort(sorter),
      otherNotes: notes.filter((n) => !n.is_pinned).sort(sorter),
    };
  }, [notes, sortMode]);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${el.scrollHeight}px`;
  }, [draftContent]);

  const openNote = useCallback(async (noteId: string) => {
    try {
      const note = await notesApi.get(noteId);
      activeNoteIdRef.current = note.id;
      setActiveNote(note);
      setDraftContent(note.content);
      setDraftTitle(note.title);
      setHasUnsavedChanges(false);
      const [bl, ol, tgs] = await Promise.all([
        notesApi.backlinks(noteId),
        notesApi.outgoingLinks(noteId),
        tagsApi.forNote(noteId),
      ]);
      setBacklinks(bl);
      setOutgoing(ol);
      setTags(tgs);
      setShowHistory(false);
      setView('notes');
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const createNote = useCallback(async () => {
    try {
      const note = await notesApi.create({ title: 'Untitled note' });
      await refreshNotes();
      await openNote(note.id);
    } catch (e) {
      setError(String(e));
    }
  }, [openNote, refreshNotes]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
        e.preventDefault();
        createNote();
      }
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'f')) {
        e.preventDefault();
        setShowSearch(true);
      }
      if (e.key === 'Escape' && showHistory) {
        setShowHistory(false);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [createNote, showHistory]);

  const onContentChange = (value: string) => {
    setDraftContent(value);
    setHasUnsavedChanges(true);
    if (!activeNote) return;
    const noteId = activeNote.id;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(async () => {
      if (activeNoteIdRef.current !== noteId) return;
      try {
        const updated = await notesApi.updateContent({
          note_id: noteId,
          content: value,
          snapshot_previous: false,
        });
        setActiveNote((prev) => (prev?.id === noteId ? updated : prev));
        setHasUnsavedChanges(false);
        setOutgoing(await notesApi.outgoingLinks(noteId));
        await refreshNotes();
      } catch (e) {
        setError(String(e));
      }
    }, 800);
  };
  const onTitleChange = (value: string) => {
    setDraftTitle(value);
    if (!activeNote) return;
    const noteId = activeNote.id;
    if (titleTimer.current) clearTimeout(titleTimer.current);
    const trimmed = value.trim();
    if (!trimmed) return;
    titleTimer.current = setTimeout(async () => {
      if (activeNoteIdRef.current !== noteId) return;
      if (trimmed === activeNote.title) return;
      try {
        const updated = await notesApi.rename(noteId, trimmed);
        setActiveNote((prev) => (prev?.id === noteId ? updated : prev));
        setDraftTitle(updated.title);
        await refreshNotes();
      } catch (e) {
        setError(String(e));
      }
    }, 500);
  };

  const onTitleBlur = async () => {
    if (titleTimer.current) clearTimeout(titleTimer.current);
    const trimmed = draftTitle.trim();
    if (!trimmed) {
      setDraftTitle(activeNote?.title ?? '');
      return;
    }
    if (!activeNote || trimmed === activeNote.title) return;
    try {
      const updated = await notesApi.rename(activeNote.id, trimmed);
      setActiveNote(updated);
      setDraftTitle(updated.title);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const togglePinned = async () => {
    if (!activeNote) return;
    try {
      const updated = await notesApi.setPinned(activeNote.id, !activeNote.is_pinned);
      setActiveNote(updated);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const takeSnapshot = async () => {
    if (!activeNote) return;
    try {
      let noteId = activeNote.id;
      if (hasUnsavedChanges) {
        if (saveTimer.current) clearTimeout(saveTimer.current);
        const updated = await notesApi.updateContent({
          note_id: noteId,
          content: draftContent,
          snapshot_previous: false,
        });
        setActiveNote(updated);
        setHasUnsavedChanges(false);
        noteId = updated.id;
      }
      await notesApi.createSnapshot(noteId);
      setSnapshotJustSaved(true);
      if (snapshotFeedbackTimer.current) clearTimeout(snapshotFeedbackTimer.current);
      snapshotFeedbackTimer.current = setTimeout(() => setSnapshotJustSaved(false), 2000);
      if (showHistory) setVersions(await notesApi.listVersions(noteId));
    } catch (e) {
      setError(String(e));
    }
  };

  const loadHistory = async () => {
    if (!activeNote) return;
    try {
      setVersions(await notesApi.listVersions(activeNote.id));
      setShowHistory(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const restoreVersion = async (versionId: string) => {
    try {
      const restored = await notesApi.restoreVersion(versionId);
      activeNoteIdRef.current = restored.id;
      setActiveNote(restored);
      setDraftContent(restored.content);
      setDraftTitle(restored.title);
      setHasUnsavedChanges(false);
      setShowHistory(false);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const addTag = async () => {
    if (!activeNote || !newTagName.trim()) return;
    try {
      await tagsApi.addToNote(activeNote.id, newTagName.trim());
      setTags(await tagsApi.forNote(activeNote.id));
      setNewTagName('');
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const removeTag = async (tagId: string) => {
    if (!activeNote) return;
    try {
      await tagsApi.removeFromNote(activeNote.id, tagId);
      setTags(await tagsApi.forNote(activeNote.id));
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const setTagColor = async (tagId: string, color: string) => {
    try {
      await tagsApi.setColor(tagId, color);
      if (activeNote) setTags(await tagsApi.forNote(activeNote.id));
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const connectNotes = async (sourceId: string, targetId: string) => {
    if (sourceId === targetId) return;
    try {
      const [source, target] = await Promise.all([notesApi.get(sourceId), notesApi.get(targetId)]);
      const linkText = target.slug === target.title ? `[[${target.title}]]` : `[[${target.slug}|${target.title}]]`;
      if (source.content.includes(linkText)) return;
      const newContent =
        source.content.trim().length > 0 ? `${source.content}\n\n${linkText}` : linkText;
      const updated = await notesApi.updateContent({
        note_id: sourceId,
        content: newContent,
        snapshot_previous: false,
      });
      if (activeNote?.id === sourceId) {
        setActiveNote(updated);
        setDraftContent(updated.content);
        setOutgoing(await notesApi.outgoingLinks(updated.id));
      }
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const createBase = async (name: string, color: string) => {
    try {
      await basesApi.create(name, color);
      await refreshNotes();
      setView('graph');
    } catch (e) {
      setError(String(e));
    }
  };

  const connectNoteToBase = async (noteId: string, baseId: string) => {
    try {
      await basesApi.addNote(baseId, noteId);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const renameBase = async (baseId: string, name: string) => {
    try {
      await basesApi.rename(baseId, name);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const setBaseColor = async (baseId: string, color: string) => {
    try {
      await basesApi.setColor(baseId, color);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const deleteBase = async (baseId: string) => {
    try {
      await basesApi.remove(baseId);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };
  const setPinnedById = async (noteId: string, pinned: boolean) => {
    try {
      const updated = await notesApi.setPinned(noteId, pinned);
      if (activeNote?.id === noteId) setActiveNote(updated);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const renameById = async (noteId: string, newTitle: string) => {
    try {
      const updated = await notesApi.rename(noteId, newTitle);
      if (activeNote?.id === noteId) {
        setActiveNote(updated);
        setDraftTitle(updated.title);
      }
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const setColorById = async (noteId: string, color: string | null) => {
    try {
      const updated = await notesApi.setColor(noteId, color);
      if (activeNote?.id === noteId) setActiveNote(updated);
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  const deleteById = async (noteId: string) => {
    try {
      await notesApi.remove(noteId);
      if (activeNote?.id === noteId) {
        activeNoteIdRef.current = null;
        setActiveNote(null);
        setDraftContent('');
        setDraftTitle('');
      }
      await refreshNotes();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="flex h-screen overflow-hidden">
      <aside className="flex w-64 shrink-0 flex-col border-r border-ink-800 bg-ink-900">
        <div className="flex items-center justify-between border-b border-ink-800 px-4 py-3">
          <div className="truncate text-sm font-medium text-zinc-200">{vault.name}</div>
          <button onClick={onCloseVault} className="shrink-0 text-xs text-zinc-500 hover:text-zinc-300">
            Switch
          </button>
        </div>

        <div className="flex gap-1.5 px-3 pt-2 pb-1">
          <button
            onClick={createNote}
            title="New note (⌘N)"
            className="flex flex-1 items-center justify-center gap-1.5 rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-dim"
          >
            <Plus size={15} /> New note
          </button>
          <button
            onClick={() => setShowCreateBase(true)}
            title="New base -- group notes together with a shared color"
            className="flex items-center justify-center gap-1 rounded-md border border-ink-700 px-2.5 py-1.5 text-sm text-zinc-400 hover:border-accent/60 hover:text-zinc-200"
          >
            <Layers size={15} />
          </button>
        </div>

        <div className="px-3 py-1.5">
          <button
            onClick={() => setShowSearch(true)}
            className="flex w-full items-center gap-2 rounded-md border border-ink-700 bg-ink-950 px-2.5 py-1.5 text-left text-xs text-zinc-500 hover:border-accent/60 hover:text-zinc-300"
          >
            <Search size={13} className="shrink-0" />
            <span className="flex-1">Search notes…</span>
            <kbd className="shrink-0 rounded border border-ink-700 px-1 py-0.5 text-[10px] text-zinc-600">⌘K</kbd>
          </button>
        </div>

        <div className="flex gap-1 px-3 pb-1.5">
          <ViewTab active={view === 'notes'} onClick={() => setView('notes')} icon={<FileText size={13} />}>
            Notes
          </ViewTab>
          <ViewTab active={view === 'graph'} onClick={() => setView('graph')} icon={<Network size={13} />}>
            Graph
          </ViewTab>
        </div>

        {view === 'notes' && notes.length > 1 && (
          <div className="flex items-center justify-end gap-1 px-3 pb-1.5">
            <button
              onClick={() => setSortMode('recent')}
              className={`rounded px-1.5 py-0.5 text-[11px] ${
                sortMode === 'recent' ? 'text-zinc-300' : 'text-zinc-600 hover:text-zinc-400'
              }`}
            >
              Recent
            </button>
            <span className="text-zinc-700">·</span>
            <button
              onClick={() => setSortMode('alphabetical')}
              className={`rounded px-1.5 py-0.5 text-[11px] ${
                sortMode === 'alphabetical' ? 'text-zinc-300' : 'text-zinc-600 hover:text-zinc-400'
              }`}
            >
              A–Z
            </button>
          </div>
        )}

        <nav className="flex-1 overflow-y-auto px-2 py-1">
          {notes.length === 0 ? (
            <p className="px-2 py-4 text-xs text-zinc-600">No notes yet. Press ⌘N to create one.</p>
          ) : (
            <>
              {pinnedNotes.length > 0 && (
                <>
                  <p className="px-2 pb-1 pt-2 text-[11px] font-medium uppercase tracking-wide text-zinc-600">
                    Pinned
                  </p>
                  {pinnedNotes.map((n) => (
                    <NoteListItem
                      key={n.id}
                      note={n}
                      color={noteColors[n.id]}
                      active={view === 'notes' && activeNote?.id === n.id}
                      onOpen={() => openNote(n.id)}
                      onContextMenu={(e) => {
                        e.preventDefault();
                        setSidebarMenu({ note: n, x: e.clientX, y: e.clientY });
                      }}
                    />
                  ))}
                  <p className="px-2 pb-1 pt-3 text-[11px] font-medium uppercase tracking-wide text-zinc-600">
                    All notes
                  </p>
                </>
              )}
              {otherNotes.map((n) => (
                <NoteListItem
                  key={n.id}
                  note={n}
                  color={noteColors[n.id]}
                  active={view === 'notes' && activeNote?.id === n.id}
                  onOpen={() => openNote(n.id)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    setSidebarMenu({ note: n, x: e.clientX, y: e.clientY });
                  }}
                />
              ))}
            </>
          )}
        </nav>

        <button
          onClick={() => setShowSettings(true)}
          className="flex shrink-0 items-center gap-2 border-t border-ink-800 px-4 py-3 text-left text-xs text-zinc-500 hover:bg-ink-800 hover:text-zinc-300"
        >
          <Settings size={14} />
          Settings
        </button>
      </aside>

      {showSettings && (
        <SettingsModal
          vault={vault}
          noteCount={notes.length}
          folderCount={folders.length}
          linkCount={allLinks.length}
          tagCount={allTags.length}
          baseCount={bases.length}
          onClose={() => setShowSettings(false)}
          onSwitchVault={() => {
            setShowSettings(false);
            onCloseVault();
          }}
        />
      )}
      {sidebarMenu && (
        <NoteContextMenu
          note={sidebarMenu.note}
          x={sidebarMenu.x}
          y={sidebarMenu.y}
          onClose={() => setSidebarMenu(null)}
          onOpen={openNote}
          onTogglePin={setPinnedById}
          onRename={renameById}
          onSetColor={setColorById}
          onDelete={deleteById}
        />
      )}

      {showSearch && (
        <SearchModal recentNotes={notes} onClose={() => setShowSearch(false)} onOpenNote={openNote} />
      )}

      {showCreateBase && <CreateBaseModal onClose={() => setShowCreateBase(false)} onCreate={createBase} />}
      <main className="flex min-h-0 flex-1 flex-col overflow-hidden">
        {error && (
          <button
            onClick={() => setError(null)}
            className="flex shrink-0 items-center justify-between bg-red-950/60 px-6 py-2 text-left text-sm text-red-300"
          >
            <span>{error}</span>
            <X size={14} className="shrink-0" />
          </button>
        )}

        {view === 'graph' ? (
          <GraphView
            notes={notes}
            links={allLinks}
            tags={allTags}
            noteTagAssignments={noteTagAssignments}
            bases={bases}
            baseNoteAssignments={baseNoteAssignments}
            noteColors={noteColors}
            selectedId={activeNote?.id ?? null}
            onOpenNote={openNote}
            onConnect={connectNotes}
            onConnectNoteToBase={connectNoteToBase}
            onTagColorChange={setTagColor}
            onTogglePin={setPinnedById}
            onRename={renameById}
            onSetColor={setColorById}
            onDelete={deleteById}
            onRenameBase={renameBase}
            onSetBaseColor={setBaseColor}
            onDeleteBase={deleteBase}
          />
        ) : !activeNote ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center">
            <p className="text-sm text-zinc-500">Select a note or create a new one.</p>
            <button
              onClick={createNote}
              className="flex items-center gap-1.5 rounded-md bg-accent px-3 py-1.5 text-sm text-white hover:bg-accent-dim"
            >
              <Plus size={14} /> New note <span className="ml-1 text-xs opacity-60">⌘N</span>
            </button>
          </div>
        ) : (
          <div className="flex flex-1 flex-col overflow-y-auto px-8 py-8">
            <div className="flex items-start gap-2">
              <input
                value={draftTitle}
                onChange={(e) => onTitleChange(e.target.value)}
                onBlur={onTitleBlur}
                className="w-full bg-transparent text-3xl font-semibold text-zinc-100 focus:outline-none"
                placeholder="Note title"
              />
              <button
                onClick={togglePinned}
                title={activeNote.is_pinned ? 'Unpin' : 'Pin'}
                className="mt-1 shrink-0 rounded-md p-1.5 text-zinc-500 hover:bg-ink-800 hover:text-accent"
              >
                {activeNote.is_pinned ? <Pin size={17} className="text-accent" /> : <PinOff size={17} />}
              </button>
            </div>
            <div className="mt-1.5 flex items-center gap-3 text-xs text-zinc-600">
              <span>{activeNote.word_count.toLocaleString()} words</span>
              <span>·</span>
              <span>Updated {formatDate(activeNote.updated_at)}</span>
              {hasUnsavedChanges && (
                <>
                  <span>·</span>
                  <span className="text-zinc-500 italic">Saving…</span>
                </>
              )}
            </div>
            <textarea
              ref={textareaRef}
              value={draftContent}
              onChange={(e) => onContentChange(e.target.value)}
              placeholder={'Start writing…\n\nUse [[Note Title]] to link to another note.'}
              className="mt-6 min-h-64 w-full resize-none overflow-hidden rounded-lg border border-ink-800 bg-ink-900/60 p-4 font-mono text-sm leading-relaxed text-zinc-200 focus:border-accent focus:outline-none"
            />
            <div className="mt-3 flex items-center gap-2">
              <button
                onClick={takeSnapshot}
                className="flex items-center gap-1.5 rounded-md border border-ink-700 px-3 py-1.5 text-xs text-zinc-400 hover:border-accent/60 hover:text-zinc-200"
              >
                {snapshotJustSaved ? <Check size={13} className="text-emerald-400" /> : <Camera size={13} />}
                Save snapshot
              </button>
              {snapshotJustSaved && (
                <span className="text-xs text-emerald-400">Saved to history</span>
              )}
              <button
                onClick={showHistory ? () => setShowHistory(false) : loadHistory}
                className="flex items-center gap-1.5 rounded-md border border-ink-700 px-3 py-1.5 text-xs text-zinc-400 hover:border-accent/60 hover:text-zinc-200"
              >
                <HistoryIcon size={13} /> {showHistory ? 'Hide history' : 'History'}
              </button>
            </div>
            {showHistory && (
              <div className="mt-3 rounded-lg border border-ink-800 bg-ink-900 p-3">
                {versions.length === 0 ? (
                  <p className="text-xs text-zinc-600">No snapshots yet — click &ldquo;Save snapshot&rdquo; to save the current version.</p>
                ) : (
                  <div className="space-y-1.5">
                    {versions.map((v) => (
                      <div key={v.id} className="flex items-center justify-between rounded-md px-2 py-1.5 text-xs hover:bg-ink-800">
                        <span className="text-zinc-400">{formatDate(v.created_at)}</span>
                        <button
                          onClick={() => restoreVersion(v.id)}
                          className="text-accent hover:underline"
                        >
                          Restore
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
            <div className="h-16 shrink-0" />
          </div>
        )}
      </main>
      {view === 'notes' && activeNote && (
        <aside className="flex w-72 shrink-0 flex-col overflow-y-auto border-l border-ink-800 bg-ink-900 px-4 py-5">
          <Section title="Tags" icon={<TagIcon size={12} />}>
            <div className="mb-2 flex flex-wrap gap-1.5">
              {tags.length === 0 ? (
                <Empty>No tags yet.</Empty>
              ) : (
                tags.map((t) => (
                  <Pill key={t.id} color={t.color} onRemove={() => removeTag(t.id)}>
                    {t.name}
                    <input
                      type="color"
                      value={t.color ?? '#7c6cf6'}
                      onChange={(e) => setTagColor(t.id, e.target.value)}
                      title="Change tag color"
                      className="h-3 w-3 cursor-pointer rounded-full border-0 bg-transparent p-0 [&::-webkit-color-swatch]:rounded-full [&::-webkit-color-swatch]:border-0 [&::-webkit-color-swatch-wrapper]:rounded-full [&::-webkit-color-swatch-wrapper]:p-0"
                    />
                  </Pill>
                ))
              )}
            </div>
            <div className="flex gap-1.5">
              <input
                value={newTagName}
                onChange={(e) => setNewTagName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addTag()}
                placeholder="Add a tag…"
                className="w-full rounded-md border border-ink-700 bg-ink-950 px-2 py-1 text-xs text-zinc-200 placeholder:text-zinc-600 focus:border-accent focus:outline-none"
              />
              <button
                onClick={addTag}
                disabled={!newTagName.trim()}
                className="shrink-0 rounded-md border border-ink-700 px-2 py-1 text-xs text-zinc-400 hover:border-accent/60 disabled:opacity-40"
              >
                <Plus size={13} />
              </button>
            </div>
          </Section>
          <Section title={`Outgoing links (${outgoing.length})`}>
            {outgoing.length === 0 ? (
              <Empty>No links yet. Write [[Note Name]] to link.</Empty>
            ) : (
              outgoing.map((l) =>
                l.target_note_id ? (
                  <button
                    key={l.id}
                    onClick={() => openNote(l.target_note_id!)}
                    className="block w-full truncate text-left text-sm text-zinc-300 hover:text-accent"
                  >
                    {l.target_text}
                  </button>
                ) : (
                  <div key={l.id} className="flex items-center gap-1 text-sm">
                    <span className="truncate text-red-400">{l.target_text}</span>
                    <span className="shrink-0 text-xs text-red-600">broken</span>
                  </div>
                ),
              )
            )}
          </Section>
          <Section title={`Backlinks (${backlinks.length})`}>
            {backlinks.length === 0 ? (
              <Empty>Nothing links here yet.</Empty>
            ) : (
              backlinks.map((b) => (
                <button
                  key={b.source_note_id}
                  onClick={() => openNote(b.source_note_id)}
                  className="block w-full truncate text-left text-sm text-zinc-300 hover:text-accent"
                >
                  {b.source_title}
                </button>
              ))
            )}
          </Section>
        </aside>
      )}
    </div>
  );
}

function ViewTab({ active, onClick, icon, children }: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1.5 text-xs font-medium transition ${
        active ? 'bg-ink-700 text-zinc-100' : 'text-zinc-500 hover:bg-ink-800 hover:text-zinc-300'
      }`}
    >
      {icon}{children}
    </button>
  );
}

function Section({ title, icon, children }: {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="mb-5">
      <h3 className="mb-2 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wide text-zinc-500">
        {icon}{title}
      </h3>
      <div className="space-y-1.5">{children}</div>
    </div>
  );
}

function Empty({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-zinc-600">{children}</p>;
}

function Pill({ children, color, onRemove }: {
  children: React.ReactNode;
  color?: string | null;
  onRemove?: () => void;
}) {
  return (
    <span
      className="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs text-zinc-200"
      style={{
        backgroundColor: color ? `${color}22` : '#272b34',
        border: `1px solid ${color ?? '#383d49'}`,
      }}
    >
      {children}
      {onRemove && (
        <button onClick={onRemove} className="text-zinc-500 hover:text-red-400">
          <X size={10} />
        </button>
      )}
    </span>
  );
}

function NoteListItem({
  note,
  color,
  active,
  onOpen,
  onContextMenu,
}: {
  note: NoteSummary;
  color?: string;
  active: boolean;
  onOpen: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}) {
  return (
    <button
      onClick={onOpen}
      onContextMenu={onContextMenu}
      className={`block w-full rounded-md px-2 py-1.5 text-left ${
        active ? 'bg-ink-700' : 'hover:bg-ink-800'
      }`}
    >
      <div className="flex items-center gap-1.5">
        {color ? (
          <span className="h-2 w-2 shrink-0 rounded-full" style={{ backgroundColor: color }} />
        ) : note.is_pinned ? (
          <Pin size={11} className="shrink-0 text-accent" />
        ) : null}
        <span className={`min-w-0 flex-1 truncate text-sm ${active ? 'text-zinc-100' : 'text-zinc-300'}`}>
          {note.title}
        </span>
        <span className="shrink-0 text-[11px] text-zinc-600">{formatRelativeTime(note.updated_at)}</span>
      </div>
      {note.content_preview && (
        <p className="mt-0.5 truncate pl-3.5 text-xs text-zinc-600">{note.content_preview}</p>
      )}
    </button>
  );
}

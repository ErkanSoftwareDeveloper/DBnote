import { invoke } from '@tauri-apps/api/core';
import type {
  Backlink,
  Base,
  BaseNoteAssignment,
  CreateNoteInput,
  Folder,
  Link,
  Note,
  NoteSummary,
  NoteTagAssignment,
  NoteVersion,
  SearchHit,
  Tag,
  UpdateNoteContentInput,
  VaultInfo,
} from './types';

// Tauri camel-cases command arguments, but nested payload fields stay snake_case.
export const vaultsApi = {
  list: () => invoke<VaultInfo[]>('list_vaults'),
  create: (name: string, path: string) => invoke<VaultInfo>('create_vault', { name, path }),
  open: (vaultId: string) => invoke<VaultInfo>('open_vault', { vaultId }),
  close: () => invoke<void>('close_vault'),
  current: () => invoke<string | null>('current_vault'),
  remove: (vaultId: string, deleteFiles: boolean) =>
    invoke<void>('delete_vault', { vaultId, deleteFiles }),
};

export const foldersApi = {
  list: () => invoke<Folder[]>('list_folders'),
  create: (name: string, parentId: string | null) => invoke<Folder>('create_folder', { name, parentId }),
  rename: (folderId: string, newName: string) => invoke<Folder>('rename_folder', { folderId, newName }),
  move: (folderId: string, newParentId: string | null) =>
    invoke<Folder>('move_folder', { folderId, newParentId }),
  remove: (folderId: string) => invoke<void>('delete_folder', { folderId }),
};

export const notesApi = {
  create: (input: CreateNoteInput) => invoke<Note>('create_note', { input }),
  get: (noteId: string) => invoke<Note>('get_note', { noteId }),
  updateContent: (input: UpdateNoteContentInput) => invoke<Note>('update_note_content', { input }),
  rename: (noteId: string, newTitle: string) => invoke<Note>('rename_note', { noteId, newTitle }),
  move: (noteId: string, folderId: string | null) => invoke<Note>('move_note', { noteId, folderId }),
  setPinned: (noteId: string, pinned: boolean) => invoke<Note>('set_note_pinned', { noteId, pinned }),
  setArchived: (noteId: string, archived: boolean) =>
    invoke<Note>('set_note_archived', { noteId, archived }),
  setProperties: (noteId: string, properties: Record<string, unknown>) =>
    invoke<Note>('set_note_properties', { noteId, properties }),
  setColor: (noteId: string, color: string | null) => invoke<Note>('set_note_color', { noteId, color }),
  remove: (noteId: string) => invoke<void>('delete_note', { noteId }),
  list: (folderId: string | null, includeArchived: boolean) =>
    invoke<NoteSummary[]>('list_notes', { folderId, includeArchived }),
  listVersions: (noteId: string) => invoke<NoteVersion[]>('list_note_versions', { noteId }),
  restoreVersion: (versionId: string) => invoke<Note>('restore_note_version', { versionId }),
  createSnapshot: (noteId: string) => invoke<NoteVersion>('create_snapshot', { noteId }),
  search: (query: string, limit: number) => invoke<SearchHit[]>('search_notes', { query, limit }),
  backlinks: (noteId: string) => invoke<Backlink[]>('get_backlinks', { noteId }),
  outgoingLinks: (noteId: string) => invoke<Link[]>('get_outgoing_links', { noteId }),
  allLinks: () => invoke<Link[]>('get_all_links'),
};

export const tagsApi = {
  list: () => invoke<Tag[]>('list_tags'),
  forNote: (noteId: string) => invoke<Tag[]>('list_tags_for_note', { noteId }),
  notesForTag: (tagId: string) => invoke<string[]>('list_notes_for_tag', { tagId }),
  allNoteTags: () => invoke<NoteTagAssignment[]>('list_all_note_tags'),
  addToNote: (noteId: string, tagName: string) => invoke<Tag>('add_tag_to_note', { noteId, tagName }),
  removeFromNote: (noteId: string, tagId: string) =>
    invoke<void>('remove_tag_from_note', { noteId, tagId }),
  setColor: (tagId: string, color: string | null) => invoke<Tag>('set_tag_color', { tagId, color }),
};

export const basesApi = {
  list: () => invoke<Base[]>('list_bases'),
  create: (name: string, color: string) => invoke<Base>('create_base', { name, color }),
  rename: (baseId: string, newName: string) => invoke<Base>('rename_base', { baseId, newName }),
  setColor: (baseId: string, color: string) => invoke<Base>('set_base_color', { baseId, color }),
  remove: (baseId: string) => invoke<void>('delete_base', { baseId }),
  addNote: (baseId: string, noteId: string) => invoke<void>('add_note_to_base', { baseId, noteId }),
  removeNote: (baseId: string, noteId: string) =>
    invoke<void>('remove_note_from_base', { baseId, noteId }),
  allBaseNotes: () => invoke<BaseNoteAssignment[]>('list_all_base_notes'),
};
export async function pickVaultDirectory(): Promise<string | null> {
  const { open } = await import('@tauri-apps/plugin-dialog');
  const result = await open({ directory: true, multiple: false, title: 'Choose a vault folder' });
  if (typeof result === 'string') return result;
  return null;
}

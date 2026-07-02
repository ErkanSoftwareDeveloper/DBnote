export interface Folder {
  id: string;
  parent_id: string | null;
  name: string;
  path: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Note {
  id: string;
  folder_id: string | null;
  title: string;
  slug: string;
  content: string;
  content_format: 'markdown' | 'richtext';
  properties: Record<string, unknown>;
  icon: string | null;
  color: string | null;
  is_pinned: boolean;
  is_archived: boolean;
  word_count: number;
  created_at: string;
  updated_at: string;
}

export interface NoteSummary {
  id: string;
  folder_id: string | null;
  title: string;
  slug: string;
  icon: string | null;
  color: string | null;
  content_preview: string;
  is_pinned: boolean;
  is_archived: boolean;
  updated_at: string;
}

export interface NoteVersion {
  id: string;
  note_id: string;
  title: string;
  content: string;
  created_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string | null;
}

export interface Base {
  id: string;
  name: string;
  color: string;
  note_count: number;
  created_at: string;
  updated_at: string;
}

export interface BaseNoteAssignment {
  base_id: string;
  note_id: string;
  base_color: string;
}

export interface NoteTagAssignment {
  note_id: string;
  tag_id: string;
  tag_name: string;
  color: string | null;
}

export interface Link {
  id: string;
  source_note_id: string;
  target_note_id: string | null;
  target_text: string;
  is_broken: boolean;
  created_at: string;
}

export interface Backlink {
  source_note_id: string;
  source_title: string;
  source_slug: string;
  target_text: string;
}

export interface SearchHit {
  note_id: string;
  title: string;
  slug: string;
  snippet: string;
  rank: number;
}

export interface VaultInfo {
  id: string;
  name: string;
  path: string;
  created_at: string;
  last_opened_at: string | null;
}

export interface CreateNoteInput {
  title: string;
  folder_id?: string | null;
  content?: string | null;
  content_format?: string | null;
}

export interface UpdateNoteContentInput {
  note_id: string;
  content: string;
  snapshot_previous: boolean;
}

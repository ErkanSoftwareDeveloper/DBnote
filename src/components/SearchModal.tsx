'use client';

import { useEffect, useRef, useState } from 'react';
import { Clock, CornerDownLeft, FileText, Search } from 'lucide-react';
import { notesApi } from '@/lib/tauri-api';
import { formatRelativeTime } from '@/lib/format';
import type { NoteSummary, SearchHit } from '@/lib/types';

export function SearchModal({
  recentNotes,
  onClose,
  onOpenNote,
}: {
    recentNotes: NoteSummary[];
  onClose: () => void;
  onOpenNote: (noteId: string) => void;
}) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchHit[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const searchTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const showingRecent = query.trim().length === 0;
  const items = showingRecent ? recentNotes.slice(0, 8) : results;

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query, results.length]);
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const active = list.children[selectedIndex] as HTMLElement | undefined;
    active?.scrollIntoView({ block: 'nearest' });
  }, [selectedIndex]);

  const onQueryChange = (value: string) => {
    setQuery(value);
    if (searchTimer.current) clearTimeout(searchTimer.current);
    if (!value.trim()) {
      setResults([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    searchTimer.current = setTimeout(async () => {
      try {
        setResults(await notesApi.search(value, 30));
      } finally {
        setLoading(false);
      }
    }, 180);
  };

  const openSelected = () => {
    const item = items[selectedIndex];
    if (!item) return;
    const id = 'note_id' in item ? item.note_id : item.id;
    onOpenNote(id);
    onClose();
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, items.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      openSelected();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/50 px-6 pt-[12vh]"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="w-full max-w-xl overflow-hidden rounded-xl border border-ink-700 bg-ink-900 shadow-2xl">
        <div className="flex items-center gap-2.5 border-b border-ink-800 px-4 py-3">
          <Search size={16} className="shrink-0 text-zinc-500" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => onQueryChange(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder="Search your notes…"
            className="w-full bg-transparent text-sm text-zinc-100 placeholder:text-zinc-600 focus:outline-none"
          />
          <kbd className="shrink-0 rounded border border-ink-700 px-1.5 py-0.5 text-[10px] text-zinc-600">esc</kbd>
        </div>

        <div ref={listRef} className="max-h-[50vh] overflow-y-auto py-1.5">
          {showingRecent && (
            <p className="flex items-center gap-1.5 px-4 pb-1 pt-1.5 text-[11px] font-medium uppercase tracking-wide text-zinc-600">
              <Clock size={11} /> Recent
            </p>
          )}

          {items.length === 0 && !loading && (
            <p className="px-4 py-6 text-center text-sm text-zinc-600">
              {showingRecent ? 'No notes yet.' : `No results for "${query}"`}
            </p>
          )}

          {items.map((item, index) => {
            const isHit = !showingRecent;
            const id = isHit ? (item as SearchHit).note_id : (item as NoteSummary).id;
            const title = item.title;
            const selected = index === selectedIndex;
            return (
              <button
                key={id}
                onMouseEnter={() => setSelectedIndex(index)}
                onClick={() => {
                  onOpenNote(id);
                  onClose();
                }}
                className={`flex w-full items-center gap-3 px-4 py-2.5 text-left ${
                  selected ? 'bg-ink-700' : ''
                }`}
              >
                <FileText size={14} className="shrink-0 text-zinc-500" />
                <div className="min-w-0 flex-1">
                  <div className="truncate text-sm text-zinc-100">{title}</div>
                  {isHit ? (
                    <div
                      className="truncate text-xs text-zinc-500 [&_mark]:rounded [&_mark]:bg-accent/30 [&_mark]:text-zinc-200 [&_mark]:px-0.5"
                      dangerouslySetInnerHTML={{ __html: (item as SearchHit).snippet }}
                    />
                  ) : (
                    (item as NoteSummary).content_preview && (
                      <div className="truncate text-xs text-zinc-500">{(item as NoteSummary).content_preview}</div>
                    )
                  )}
                </div>
                {!isHit && (
                  <span className="shrink-0 text-xs text-zinc-600">
                    {formatRelativeTime((item as NoteSummary).updated_at)}
                  </span>
                )}
                {selected && <CornerDownLeft size={12} className="shrink-0 text-zinc-600" />}
              </button>
            );
          })}
        </div>

        <div className="flex items-center gap-3 border-t border-ink-800 px-4 py-2 text-[11px] text-zinc-600">
          <span className="flex items-center gap-1">
            <kbd className="rounded border border-ink-700 px-1 py-0.5">↑↓</kbd> navigate
          </span>
          <span className="flex items-center gap-1">
            <kbd className="rounded border border-ink-700 px-1 py-0.5">↵</kbd> open
          </span>
        </div>
      </div>
    </div>
  );
}

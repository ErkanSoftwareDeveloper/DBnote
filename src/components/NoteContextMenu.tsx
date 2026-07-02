'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { AlertTriangle, FileText, Palette, Pencil, Pin, PinOff, Trash2 } from 'lucide-react';
import type { NoteSummary } from '@/lib/types';

export function NoteContextMenu({
  note,
  x,
  y,
  onClose,
  onOpen,
  onTogglePin,
  onRename,
  onSetColor,
  onDelete,
}: {
  note: NoteSummary;
  x: number;
  y: number;
  onClose: () => void;
  onOpen: (id: string) => void;
  onTogglePin: (id: string, pinned: boolean) => void;
  onRename: (id: string, title: string) => void;
  onSetColor: (id: string, color: string | null) => void;
  onDelete: (id: string) => void;
}) {
  const [mode, setMode] = useState<'menu' | 'rename' | 'confirm-delete'>('menu');
  const [titleDraft, setTitleDraft] = useState(note.title);
  const ref = useRef<HTMLDivElement>(null);

  const submitRename = useCallback(() => {
    const trimmed = titleDraft.trim();
    if (trimmed && trimmed !== note.title) onRename(note.id, trimmed);
    onClose();
  }, [note.id, note.title, onClose, onRename, titleDraft]);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (!ref.current || ref.current.contains(e.target as Node)) return;
      submitRename();
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('mousedown', handleClick);
    window.addEventListener('keydown', handleKey);
    return () => {
      window.removeEventListener('mousedown', handleClick);
      window.removeEventListener('keydown', handleKey);
    };
  }, [onClose, submitRename]);

  const style: React.CSSProperties = {
    position: 'fixed',
    top: Math.min(y, window.innerHeight - 240),
    left: Math.min(x, window.innerWidth - 210),
    zIndex: 50,
  };

  return (
    <div ref={ref} style={style} className="w-52 rounded-lg border border-ink-700 bg-ink-900 p-1 text-sm shadow-2xl">
      {mode === 'rename' ? (
        <div className="p-1">
          <input
            autoFocus
            value={titleDraft}
            onChange={(e) => setTitleDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitRename();
              if (e.key === 'Escape') onClose();
            }}
            onBlur={submitRename}
            className="w-full rounded-md border border-ink-700 bg-ink-950 px-2 py-1.5 text-xs text-zinc-100 focus:border-accent focus:outline-none"
          />
          <p className="mt-1 px-1 text-xs text-zinc-600">Enter to save · Esc to cancel</p>
        </div>
      ) : mode === 'confirm-delete' ? (
        <div className="p-2 space-y-2">
          <div className="flex items-start gap-2 text-xs text-zinc-300">
            <AlertTriangle size={14} className="mt-0.5 shrink-0 text-red-400" />
            <span>Delete &ldquo;{note.title}&rdquo;? This cannot be undone.</span>
          </div>
          <div className="flex gap-1.5">
            <button
              onClick={() => {
                onDelete(note.id);
                onClose();
              }}
              className="flex-1 rounded-md bg-red-900/60 px-2 py-1.5 text-xs font-medium text-red-200 hover:bg-red-900"
            >
              Delete
            </button>
            <button
              onClick={() => setMode('menu')}
              className="flex-1 rounded-md border border-ink-700 px-2 py-1.5 text-xs text-zinc-400 hover:bg-ink-800"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <>
          <MenuItem icon={<FileText size={14} />} onClick={() => { onOpen(note.id); onClose(); }}>
            Open
          </MenuItem>
          <MenuItem icon={<Pencil size={14} />} onClick={() => setMode('rename')}>
            Rename
          </MenuItem>
          <MenuItem
            icon={note.is_pinned ? <PinOff size={14} /> : <Pin size={14} />}
            onClick={() => { onTogglePin(note.id, !note.is_pinned); onClose(); }}
          >
            {note.is_pinned ? 'Unpin' : 'Pin'}
          </MenuItem>
          <div className="flex items-center justify-between rounded-md px-2 py-1.5 text-zinc-300">
            <span className="flex items-center gap-2 text-sm">
              <Palette size={14} /> Color
            </span>
            <div className="flex items-center gap-1.5">
              <input
                type="color"
                value={note.color ?? '#7c6cf6'}
                onChange={(e) => onSetColor(note.id, e.target.value)}
                title="Set this note's color"
                className="h-3.5 w-3.5 cursor-pointer rounded-full border-0 bg-transparent p-0 [&::-webkit-color-swatch]:rounded-full [&::-webkit-color-swatch]:border-0 [&::-webkit-color-swatch-wrapper]:rounded-full [&::-webkit-color-swatch-wrapper]:p-0"
              />
              {note.color && (
                <button
                  onClick={() => { onSetColor(note.id, null); onClose(); }}
                  title="Clear color override"
                  className="text-xs text-zinc-500 hover:text-red-400"
                >
                  ×
                </button>
              )}
            </div>
          </div>
          <div className="my-1 border-t border-ink-800" />
          <MenuItem icon={<Trash2 size={14} />} danger onClick={() => setMode('confirm-delete')}>
            Delete
          </MenuItem>
        </>
      )}
    </div>
  );
}

function MenuItem({
  icon,
  children,
  onClick,
  danger,
}: {
  icon: React.ReactNode;
  children: React.ReactNode;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm ${
        danger ? 'text-red-400 hover:bg-red-950/40' : 'text-zinc-300 hover:bg-ink-800'
      }`}
    >
      {icon}
      {children}
    </button>
  );
}

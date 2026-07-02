'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { AlertTriangle, Layers, Palette, Pencil, Trash2 } from 'lucide-react';
import type { Base } from '@/lib/types';

export function BaseContextMenu({
  base,
  x,
  y,
  onClose,
  onRename,
  onSetColor,
  onDelete,
}: {
  base: Base;
  x: number;
  y: number;
  onClose: () => void;
  onRename: (baseId: string, name: string) => void;
  onSetColor: (baseId: string, color: string) => void;
  onDelete: (baseId: string) => void;
}) {
  const [mode, setMode] = useState<'menu' | 'rename' | 'confirm-delete'>('menu');
  const [nameDraft, setNameDraft] = useState(base.name);
  const ref = useRef<HTMLDivElement>(null);

  const submitRename = useCallback(() => {
    const trimmed = nameDraft.trim();
    if (trimmed && trimmed !== base.name) onRename(base.id, trimmed);
    onClose();
  }, [base.id, base.name, nameDraft, onClose, onRename]);

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
    top: Math.min(y, window.innerHeight - 220),
    left: Math.min(x, window.innerWidth - 210),
    zIndex: 50,
  };

  return (
    <div ref={ref} style={style} className="w-52 rounded-lg border border-ink-700 bg-ink-900 p-1 text-sm shadow-2xl">
      {mode === 'rename' ? (
        <div className="p-1">
          <input
            autoFocus
            value={nameDraft}
            onChange={(e) => setNameDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') submitRename();
              if (e.key === 'Escape') onClose();
            }}
            onBlur={submitRename}
            className="w-full rounded-md border border-ink-700 bg-ink-950 px-2 py-1.5 text-xs text-zinc-100 focus:border-accent focus:outline-none"
          />
        </div>
      ) : mode === 'confirm-delete' ? (
        <div className="space-y-2 p-2">
          <div className="flex items-start gap-2 text-xs text-zinc-300">
            <AlertTriangle size={14} className="mt-0.5 shrink-0 text-red-400" />
            <span>
              Delete base &ldquo;{base.name}&rdquo;? Its {base.note_count} note{base.note_count === 1 ? '' : 's'}{' '}
              won&apos;t be deleted, just unlinked from it.
            </span>
          </div>
          <div className="flex gap-1.5">
            <button
              onClick={() => {
                onDelete(base.id);
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
          <div className="flex items-center gap-2 px-2 py-1.5 text-xs text-zinc-500">
            <Layers size={13} />
            {base.note_count} note{base.note_count === 1 ? '' : 's'} linked
          </div>
          <div className="my-1 border-t border-ink-800" />
          <MenuItem icon={<Pencil size={14} />} onClick={() => setMode('rename')}>
            Rename
          </MenuItem>
          <div className="flex items-center justify-between rounded-md px-2 py-1.5 text-zinc-300">
            <span className="flex items-center gap-2 text-sm">
              <Palette size={14} /> Color
            </span>
            <input
              type="color"
              value={base.color}
              onChange={(e) => onSetColor(base.id, e.target.value)}
              title="Change base color -- every linked note follows"
              className="h-3.5 w-3.5 cursor-pointer rounded-full border-0 bg-transparent p-0 [&::-webkit-color-swatch]:rounded-full [&::-webkit-color-swatch]:border-0 [&::-webkit-color-swatch-wrapper]:rounded-full [&::-webkit-color-swatch-wrapper]:p-0"
            />
          </div>
          <div className="my-1 border-t border-ink-800" />
          <MenuItem icon={<Trash2 size={14} />} danger onClick={() => setMode('confirm-delete')}>
            Delete base
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

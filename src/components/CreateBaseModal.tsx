'use client';

import { useEffect, useRef, useState } from 'react';
import { Layers, X } from 'lucide-react';

const SUGGESTED_COLORS = ['#f97316', '#22c55e', '#3b82f6', '#ec4899', '#eab308', '#a855f7', '#06b6d4', '#ef4444'];

export function CreateBaseModal({
  onClose,
  onCreate,
}: {
  onClose: () => void;
  onCreate: (name: string, color: string) => void;
}) {
  const [name, setName] = useState('');
  const [color, setColor] = useState(SUGGESTED_COLORS[0]!);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [onClose]);

  const submit = () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    onCreate(trimmed, color);
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="w-full max-w-sm rounded-xl border border-ink-700 bg-ink-900 shadow-2xl">
        <div className="flex items-center justify-between border-b border-ink-800 px-5 py-4">
          <h2 className="flex items-center gap-2 text-sm font-semibold text-zinc-100">
            <Layers size={15} /> New base
          </h2>
          <button onClick={onClose} className="rounded-md p-1 text-zinc-500 hover:bg-ink-800 hover:text-zinc-300">
            <X size={16} />
          </button>
        </div>

        <div className="space-y-4 px-5 py-4">
          <p className="text-xs text-zinc-500">
            A base groups notes together. Link notes to it in the graph view (Shift+drag a note onto it), and
            every linked note follows whatever color you give the base.
          </p>

          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-400">Name</label>
            <input
              ref={inputRef}
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && submit()}
              placeholder="e.g. Work, Recipes, Project X"
              className="w-full rounded-md border border-ink-700 bg-ink-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-accent focus:outline-none"
            />
          </div>

          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-400">Color</label>
            <div className="flex flex-wrap gap-2">
              {SUGGESTED_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setColor(c)}
                  style={{ backgroundColor: c }}
                  className={`h-7 w-7 rounded-full transition ${
                    color === c ? 'ring-2 ring-white ring-offset-2 ring-offset-ink-900' : 'opacity-80 hover:opacity-100'
                  }`}
                  aria-label={`Use color ${c}`}
                />
              ))}
              <input
                type="color"
                value={color}
                onChange={(e) => setColor(e.target.value)}
                title="Pick a custom color"
                className="h-7 w-7 cursor-pointer rounded-full border-0 bg-transparent p-0 [&::-webkit-color-swatch]:rounded-full [&::-webkit-color-swatch]:border-2 [&::-webkit-color-swatch]:border-ink-700 [&::-webkit-color-swatch-wrapper]:rounded-full [&::-webkit-color-swatch-wrapper]:p-0"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 border-t border-ink-800 px-5 py-3">
          <button onClick={onClose} className="rounded-md px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-200">
            Cancel
          </button>
          <button
            onClick={submit}
            disabled={!name.trim()}
            className="rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-dim disabled:cursor-not-allowed disabled:opacity-40"
          >
            Create base
          </button>
        </div>
      </div>
    </div>
  );
}

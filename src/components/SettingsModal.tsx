'use client';

import { useEffect, useRef } from 'react';
import { FolderOpen, Hash, Keyboard, Link2, Repeat, X } from 'lucide-react';
import type { VaultInfo } from '@/lib/types';

const APP_VERSION = '0.1.0';

export function SettingsModal({
  vault,
  noteCount,
  folderCount,
  linkCount,
  tagCount,
  baseCount,
  onClose,
  onSwitchVault,
}: {
  vault: VaultInfo;
  noteCount: number;
  folderCount: number;
  linkCount: number;
  tagCount: number;
  baseCount: number;
  onClose: () => void;
  onSwitchVault: () => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={panelRef}
        className="w-full max-w-md rounded-xl border border-ink-700 bg-ink-900 shadow-2xl"
      >
        <div className="flex items-center justify-between border-b border-ink-800 px-5 py-4">
          <h2 className="text-sm font-semibold text-zinc-100">Settings</h2>
          <button onClick={onClose} className="rounded-md p-1 text-zinc-500 hover:bg-ink-800 hover:text-zinc-300">
            <X size={16} />
          </button>
        </div>

        <div className="max-h-[70vh] space-y-5 overflow-y-auto px-5 py-4">
          <section>
            <SectionLabel icon={<FolderOpen size={12} />}>Vault</SectionLabel>
            <div className="rounded-md border border-ink-800 bg-ink-950 px-3 py-2.5">
              <p className="text-sm font-medium text-zinc-100">{vault.name}</p>
              <p className="mt-0.5 break-all text-xs text-zinc-500">{vault.path}</p>
            </div>
            <button
              onClick={onSwitchVault}
              className="mt-2 flex items-center gap-1.5 text-xs text-accent hover:underline"
            >
              <Repeat size={12} /> Switch to a different vault
            </button>
          </section>
          <section>
            <SectionLabel icon={<Hash size={12} />}>Stats</SectionLabel>
            <div className="grid grid-cols-2 gap-2">
              <StatBox label="Notes" value={noteCount} />
              <StatBox label="Folders" value={folderCount} />
              <StatBox label="Links" value={linkCount} />
              <StatBox label="Tags" value={tagCount} />
              <StatBox label="Bases" value={baseCount} />
            </div>
          </section>
          <section>
            <SectionLabel icon={<Keyboard size={12} />}>Keyboard shortcuts</SectionLabel>
            <div className="space-y-1.5 rounded-md border border-ink-800 bg-ink-950 px-3 py-2.5">
              <ShortcutRow keys="⌘ N" label="New note" />
              <ShortcutRow keys="⌘ K" label="Search" />
              <ShortcutRow keys="Esc" label="Clear search / close panel" />
              <ShortcutRow keys="Shift + drag" label="Link a note to another note or a base (in Graph view)" />
              <ShortcutRow keys="Right-click" label="Note or base options" />
            </div>
          </section>
          <section className="flex items-center justify-between border-t border-ink-800 pt-3 text-xs text-zinc-600">
            <span className="flex items-center gap-1.5">
              <Link2 size={12} /> NoteDB
            </span>
            <span>v{APP_VERSION}</span>
          </section>
        </div>
      </div>
    </div>
  );
}

function SectionLabel({ icon, children }: { icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <h3 className="mb-2 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wide text-zinc-500">
      {icon}
      {children}
    </h3>
  );
}

function StatBox({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-md border border-ink-800 bg-ink-950 px-3 py-2">
      <p className="text-lg font-semibold text-zinc-100">{value.toLocaleString()}</p>
      <p className="text-xs text-zinc-500">{label}</p>
    </div>
  );
}

function ShortcutRow({ keys, label }: { keys: string; label: string }) {
  return (
    <div className="flex items-center justify-between text-xs">
      <span className="text-zinc-400">{label}</span>
      <kbd className="rounded border border-ink-700 bg-ink-800 px-1.5 py-0.5 font-mono text-zinc-300">{keys}</kbd>
    </div>
  );
}

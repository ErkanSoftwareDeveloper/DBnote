'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceSimulation,
  type Simulation,
  type SimulationLinkDatum,
  type SimulationNodeDatum,
} from 'd3-force';
import { Filter, Maximize2, Network, Palette, Tag as TagIcon } from 'lucide-react';
import { BaseContextMenu } from './BaseContextMenu';
import { NoteContextMenu } from './NoteContextMenu';
import { computeAutoColors } from '@/lib/graph-utils';
import type { Base, BaseNoteAssignment, Link, NoteSummary, NoteTagAssignment, Tag } from '@/lib/types';

interface GraphNode extends SimulationNodeDatum {
  id: string;
  title: string;
  kind: 'note' | 'base';
  connections: number;
  color?: string;
}

type GraphEdge = SimulationLinkDatum<GraphNode>;

const DEFAULT_NODE_COLOR = '#7c6cf6';
const BASE_MIN_RADIUS = 20;

function baseRadius(noteCount: number): number {
  return BASE_MIN_RADIUS + Math.sqrt(noteCount) * 7;
}

function noteRadius(connections: number, maxConnections: number): number {
  return 6 + (connections / Math.max(1, maxConnections)) * 14;
}

export function GraphView({
  notes,
  links,
  tags,
  noteTagAssignments,
  bases,
  baseNoteAssignments,
  noteColors,
  selectedId,
  onOpenNote,
  onConnect,
  onConnectNoteToBase,
  onTagColorChange,
  onTogglePin,
  onRename,
  onSetColor,
  onDelete,
  onRenameBase,
  onSetBaseColor,
  onDeleteBase,
}: {
  notes: NoteSummary[];
  links: Link[];
  tags: Tag[];
  noteTagAssignments: NoteTagAssignment[];
  bases: Base[];
  baseNoteAssignments: BaseNoteAssignment[];
  noteColors: Record<string, string | undefined>;
  selectedId?: string | null;
  onOpenNote: (noteId: string) => void;
  onConnect: (sourceId: string, targetId: string) => void;
  onConnectNoteToBase: (noteId: string, baseId: string) => void;
  onTagColorChange: (tagId: string, color: string) => void;
  onTogglePin: (noteId: string, pinned: boolean) => void;
  onRename: (noteId: string, title: string) => void;
  onSetColor: (noteId: string, color: string | null) => void;
  onDelete: (noteId: string) => void;
  onRenameBase: (baseId: string, name: string) => void;
  onSetBaseColor: (baseId: string, color: string) => void;
  onDeleteBase: (baseId: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 800, height: 600 });
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [transform, setTransform] = useState({ x: 0, y: 0, k: 1 });
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [drag, setDrag] = useState<{ id: string; kind: 'note' | 'base'; mode: 'move' | 'connect' } | null>(null);
  const [connectCursor, setConnectCursor] = useState<{ x: number; y: number } | null>(null);
  const [connectTargetId, setConnectTargetId] = useState<string | null>(null);
  // The ref is the source of truth on pointer-up; state is only for highlighting.
  const connectTargetRef = useRef<GraphNode | null>(null);
  const [selectedTagIds, setSelectedTagIds] = useState<Set<string>>(new Set());
  const [onlyConnected, setOnlyConnected] = useState(false);
  const [noteContextMenu, setNoteContextMenu] = useState<{ note: NoteSummary; x: number; y: number } | null>(null);
  const [baseContextMenu, setBaseContextMenu] = useState<{ base: Base; x: number; y: number } | null>(null);
  const panState = useRef<{ active: boolean; lastX: number; lastY: number }>({
    active: false,
    lastX: 0,
    lastY: 0,
  });
  const simulationRef = useRef<Simulation<GraphNode, GraphEdge> | null>(null);
  const nodesRef = useRef<GraphNode[]>([]);
  // Preserve settled positions across refreshes so graph edits do not reshuffle the whole canvas.
  const positionsRef = useRef<Map<string, { x: number; y: number }>>(new Map());

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const tagsByNote = useMemo(() => {
    const map = new Map<string, Set<string>>();
    for (const a of noteTagAssignments) {
      if (!map.has(a.note_id)) map.set(a.note_id, new Set());
      map.get(a.note_id)!.add(a.tag_id);
    }
    return map;
  }, [noteTagAssignments]);

  const tagNoteCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const a of noteTagAssignments) {
      counts.set(a.tag_id, (counts.get(a.tag_id) ?? 0) + 1);
    }
    return counts;
  }, [noteTagAssignments]);
  const connectionCountAll = useMemo(() => {
    const map = new Map<string, number>();
    for (const link of links) {
      map.set(link.source_note_id, (map.get(link.source_note_id) ?? 0) + 1);
      if (link.target_note_id) {
        map.set(link.target_note_id, (map.get(link.target_note_id) ?? 0) + 1);
      }
    }
    for (const a of baseNoteAssignments) {
      map.set(a.note_id, (map.get(a.note_id) ?? 0) + 1);
    }
    return map;
  }, [links, baseNoteAssignments]);

  const visibleNotes = useMemo(() => {
    return notes.filter((n) => {
      if (onlyConnected && (connectionCountAll.get(n.id) ?? 0) === 0) return false;
      if (selectedTagIds.size > 0) {
        const noteTagIds = tagsByNote.get(n.id);
        if (!noteTagIds || ![...selectedTagIds].some((id) => noteTagIds.has(id))) return false;
      }
      return true;
    });
  }, [notes, onlyConnected, selectedTagIds, tagsByNote, connectionCountAll]);
  const autoColors = useMemo(
    () => computeAutoColors(visibleNotes, links, bases, baseNoteAssignments),
    [visibleNotes, links, bases, baseNoteAssignments],
  );
  const resolvedColor = (noteId: string): string => {
    return noteColors[noteId] ?? autoColors[noteId] ?? DEFAULT_NODE_COLOR;
  };

  useEffect(() => {
    const visibleNoteIds = new Set(visibleNotes.map((n) => n.id));
    const baseIds = new Set(bases.map((b) => b.id));
    const visibleIds = new Set<string>([...visibleNoteIds, ...baseIds]);
    for (const id of positionsRef.current.keys()) {
      if (!notes.some((n) => n.id === id) && !bases.some((b) => b.id === id)) {
        positionsRef.current.delete(id);
      }
    }

    const noteNodes: GraphNode[] = visibleNotes.map((n) => {
      const prior = positionsRef.current.get(n.id);
      return {
        id: n.id,
        title: n.title,
        kind: 'note',
        connections: connectionCountAll.get(n.id) ?? 0,
        x: prior?.x,
        y: prior?.y,
      };
    });

    const baseNodes: GraphNode[] = bases.map((b) => {
      const prior = positionsRef.current.get(b.id);
      return {
        id: b.id,
        title: b.name,
        kind: 'base',
        connections: b.note_count,
        color: b.color,
        x: prior?.x,
        y: prior?.y,
      };
    });

    const simNodes: GraphNode[] = [...noteNodes, ...baseNodes];

    const noteLinkEdges: GraphEdge[] = links
      .filter((l) => l.target_note_id && visibleIds.has(l.source_note_id) && visibleIds.has(l.target_note_id))
      .map((l) => ({ source: l.source_note_id, target: l.target_note_id as string }));

    const baseLinkEdges: GraphEdge[] = baseNoteAssignments
      .filter((a) => visibleIds.has(a.note_id) && visibleIds.has(a.base_id))
      .map((a) => ({ source: a.note_id, target: a.base_id }));

    const simEdges: GraphEdge[] = [...noteLinkEdges, ...baseLinkEdges];

    nodesRef.current = simNodes;

    const simulation = forceSimulation(simNodes)
      .force(
        'link',
        forceLink<GraphNode, GraphEdge>(simEdges)
          .id((d) => d.id)
          .distance((d) => {
            const target = d.target as GraphNode;
            return target.kind === 'base' ? 140 : 110;
          }),
      )
      .force('charge', forceManyBody<GraphNode>().strength((d) => (d.kind === 'base' ? -420 : -260)))
      .force('center', forceCenter(size.width / 2, size.height / 2))
      .force(
        'collide',
        forceCollide<GraphNode>((d) => (d.kind === 'base' ? baseRadius(d.connections) + 14 : 34)),
      )
      .on('tick', () => {
        for (const n of simNodes) {
          if (n.x != null && n.y != null) positionsRef.current.set(n.id, { x: n.x, y: n.y });
        }
        setNodes([...simNodes]);
        setEdges([...simEdges]);
      });

    simulationRef.current = simulation;

    return () => {
      simulation.stop();
    };
  }, [visibleNotes, notes, links, bases, baseNoteAssignments, connectionCountAll, size.width, size.height]);

  const toGraphSpace = useCallback((clientX: number, clientY: number) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return {
      x: (clientX - rect.left - transform.x) / transform.k,
      y: (clientY - rect.top - transform.y) / transform.k,
    };
  }, [transform.k, transform.x, transform.y]);

  const maxConnections = useMemo(
    () => Math.max(1, ...nodes.filter((n) => n.kind === 'note').map((n) => n.connections)),
    [nodes],
  );

  const hitRadius = useCallback(
    (n: GraphNode): number =>
      (n.kind === 'base' ? baseRadius(n.connections) : noteRadius(n.connections, maxConnections)) + 10,
    [maxConnections],
  );

  const nodeAt = useCallback((x: number, y: number, excludeId?: string): GraphNode | null => {
    let closest: GraphNode | null = null;
    let closestDist = Infinity;
    for (const n of nodesRef.current) {
      if (n.id === excludeId || n.x == null || n.y == null) continue;
      const dist = Math.hypot(n.x - x, n.y - y);
      if (dist < hitRadius(n) && dist < closestDist) {
        closest = n;
        closestDist = dist;
      }
    }
    return closest;
  }, [hitRadius]);

  const startDrag = (node: GraphNode) => (e: React.PointerEvent) => {
    e.stopPropagation();
    const mode = e.shiftKey ? 'connect' : 'move';
    setDrag({ id: node.id, kind: node.kind, mode });
    if (mode === 'move') {
      node.fx = node.x;
      node.fy = node.y;
      simulationRef.current?.alphaTarget(0.3).restart();
    } else {
      simulationRef.current?.stop();
    }
  };

  useEffect(() => {
    if (!drag) return;

    const handleMove = (e: PointerEvent) => {
      const { x, y } = toGraphSpace(e.clientX, e.clientY);
      if (drag.mode === 'move') {
        const node = nodesRef.current.find((n) => n.id === drag.id);
        if (node) {
          node.fx = x;
          node.fy = y;
          setNodes([...nodesRef.current]);
        }
      } else {
        setConnectCursor({ x, y });
        const target = nodeAt(x, y, drag.id);
        const valid = target && !(drag.kind === 'base' && target.kind === 'base');
        connectTargetRef.current = valid ? target : null;
        setConnectTargetId(valid ? target!.id : null);
      }
    };

    const handleUp = () => {
      if (drag.mode === 'move') {
        const node = nodesRef.current.find((n) => n.id === drag.id);
        if (node) {
          node.fx = undefined;
          node.fy = undefined;
        }
        simulationRef.current?.alphaTarget(0);
      } else {
        const target = connectTargetRef.current;
        if (target && !(drag.kind === 'base' && target.kind === 'base')) {
          if (drag.kind === 'note' && target.kind === 'note') {
            onConnect(drag.id, target.id);
          } else if (drag.kind === 'note' && target.kind === 'base') {
            onConnectNoteToBase(drag.id, target.id);
          } else if (drag.kind === 'base' && target.kind === 'note') {
            onConnectNoteToBase(target.id, drag.id);
          }
        }
        simulationRef.current?.restart();
      }
      connectTargetRef.current = null;
      setConnectCursor(null);
      setConnectTargetId(null);
      setDrag(null);
    };

    window.addEventListener('pointermove', handleMove);
    window.addEventListener('pointerup', handleUp);
    return () => {
      window.removeEventListener('pointermove', handleMove);
      window.removeEventListener('pointerup', handleUp);
    };
  }, [drag, nodeAt, onConnect, onConnectNoteToBase, toGraphSpace]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
      setTransform((prev) => ({
        ...prev,
        k: Math.min(2.5, Math.max(0.3, prev.k * (e.deltaY < 0 ? 1.08 : 0.93))),
      }));
    };
    el.addEventListener('wheel', handleWheel, { passive: false });
    return () => el.removeEventListener('wheel', handleWheel);
  }, []);

  const onBackgroundPointerDown = (e: React.PointerEvent) => {
    panState.current = { active: true, lastX: e.clientX, lastY: e.clientY };
  };
  const onBackgroundPointerMove = (e: React.PointerEvent) => {
    if (!panState.current.active) return;
    const dx = e.clientX - panState.current.lastX;
    const dy = e.clientY - panState.current.lastY;
    panState.current.lastX = e.clientX;
    panState.current.lastY = e.clientY;
    setTransform((prev) => ({ ...prev, x: prev.x + dx, y: prev.y + dy }));
  };
  const onBackgroundPointerUp = () => {
    panState.current.active = false;
  };

  const resetView = () => setTransform({ x: 0, y: 0, k: 1 });

  const toggleTagFilter = (tagId: string) => {
    setSelectedTagIds((prev) => {
      const next = new Set(prev);
      if (next.has(tagId)) next.delete(tagId);
      else next.add(tagId);
      return next;
    });
  };

  const clearFilters = () => {
    setSelectedTagIds(new Set());
    setOnlyConnected(false);
  };

  const focusId = hoveredId ?? drag?.id ?? null;
  const focusedNeighbors = useMemo(() => {
    if (!focusId) return null;
    const neighbors = new Set<string>([focusId]);
    for (const e of edges) {
      const s = typeof e.source === 'object' ? e.source.id : e.source;
      const t = typeof e.target === 'object' ? e.target.id : e.target;
      if (s === focusId) neighbors.add(t as string);
      if (t === focusId) neighbors.add(s as string);
    }
    return neighbors;
  }, [focusId, edges]);

  const gridSize = 26 * transform.k;
  const filtersActive = selectedTagIds.size > 0 || onlyConnected;

  return (
    <div className="flex h-full w-full">
      <div
        ref={containerRef}
        className="relative h-full flex-1 cursor-grab overflow-hidden bg-ink-950 active:cursor-grabbing"
        style={{
          backgroundImage: 'radial-gradient(circle, #20232b 1.4px, transparent 1.4px)',
          backgroundSize: `${gridSize}px ${gridSize}px`,
          backgroundPosition: `${transform.x % gridSize}px ${transform.y % gridSize}px`,
        }}
        onPointerDown={onBackgroundPointerDown}
        onPointerMove={onBackgroundPointerMove}
        onPointerUp={onBackgroundPointerUp}
        onPointerLeave={onBackgroundPointerUp}
      >
        {visibleNotes.length === 0 && bases.length === 0 ? (
          <div className="flex h-full items-center justify-center px-6 text-center text-sm text-zinc-600">
            Create a couple of notes and link them with [[Note Title]], or create a Base to group notes
            together, to see the graph.
          </div>
        ) : (
          <svg width="100%" height="100%">
            <defs>
              <filter id="node-glow" x="-60%" y="-60%" width="220%" height="220%">
                <feDropShadow dx="0" dy="0" stdDeviation="3" floodOpacity="0.55" />
              </filter>
            </defs>
            <g transform={`translate(${transform.x}, ${transform.y}) scale(${transform.k})`}>
              {edges.map((e, i) => {
                const source = e.source as GraphNode;
                const target = e.target as GraphNode;
                if (typeof source !== 'object' || typeof target !== 'object') return null;
                const dimmed =
                  focusedNeighbors && !(focusedNeighbors.has(source.id) && focusedNeighbors.has(target.id));
                const isBaseEdge = source.kind === 'base' || target.kind === 'base';
                const baseNode = source.kind === 'base' ? source : target.kind === 'base' ? target : null;
                return (
                  <line
                    key={i}
                    x1={source.x ?? 0}
                    y1={source.y ?? 0}
                    x2={target.x ?? 0}
                    y2={target.y ?? 0}
                    stroke={isBaseEdge ? baseNode?.color ?? '#454b59' : '#454b59'}
                    strokeWidth={isBaseEdge ? 2 : 1.5}
                    opacity={dimmed ? 0.12 : isBaseEdge ? 0.55 : 0.85}
                  />
                );
              })}

              {connectCursor && drag?.mode === 'connect' && (
                <line
                  x1={nodesRef.current.find((n) => n.id === drag.id)?.x ?? 0}
                  y1={nodesRef.current.find((n) => n.id === drag.id)?.y ?? 0}
                  x2={
                    connectTargetId
                      ? nodesRef.current.find((n) => n.id === connectTargetId)?.x ?? connectCursor.x
                      : connectCursor.x
                  }
                  y2={
                    connectTargetId
                      ? nodesRef.current.find((n) => n.id === connectTargetId)?.y ?? connectCursor.y
                      : connectCursor.y
                  }
                  stroke={connectTargetId ? '#22c55e' : '#7c6cf6'}
                  strokeWidth={2}
                  strokeDasharray="6 5"
                />
              )}

              {nodes.map((n) => {
                const isBase = n.kind === 'base';
                const radius = isBase ? baseRadius(n.connections) : noteRadius(n.connections, maxConnections);
                const color = isBase ? n.color ?? DEFAULT_NODE_COLOR : resolvedColor(n.id);
                const dimmed = focusedNeighbors && !focusedNeighbors.has(n.id);
                const isSelected = selectedId === n.id;
                const isFocused = focusId === n.id;
                const isConnectTarget = connectTargetId === n.id;
                return (
                  <g
                    key={n.id}
                    transform={`translate(${n.x ?? 0}, ${n.y ?? 0})`}
                    onPointerDown={startDrag(n)}
                    onPointerEnter={() => setHoveredId(n.id)}
                    onPointerLeave={() => setHoveredId((id) => (id === n.id ? null : id))}
                    onDoubleClick={() => {
                      if (!isBase) onOpenNote(n.id);
                    }}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      if (isBase) {
                        const baseData = bases.find((b) => b.id === n.id);
                        if (baseData) setBaseContextMenu({ base: baseData, x: e.clientX, y: e.clientY });
                      } else {
                        const summary = notes.find((note) => note.id === n.id);
                        if (summary) setNoteContextMenu({ note: summary, x: e.clientX, y: e.clientY });
                      }
                    }}
                    className="cursor-pointer"
                    opacity={dimmed ? 0.25 : 1}
                    style={{ transition: 'opacity 150ms ease' }}
                  >
                    {isConnectTarget && (
                      <circle r={radius + 8} fill="none" stroke="#22c55e" strokeWidth={2} />
                    )}
                    {isSelected && !isConnectTarget && (
                      <circle r={radius + 6} fill="none" stroke="#7c6cf6" strokeWidth={1.5} strokeDasharray="3 3" />
                    )}
                    {isBase ? (
                      <rect
                        x={-radius * 0.78}
                        y={-radius * 0.78}
                        width={radius * 1.56}
                        height={radius * 1.56}
                        rx={radius * 0.22}
                        fill={color}
                        fillOpacity={isFocused ? 1 : 0.9}
                        stroke="#0b0c0f"
                        strokeWidth={2.5}
                        transform="rotate(45)"
                        filter={isFocused ? 'url(#node-glow)' : undefined}
                        style={{ transition: 'width 1200ms ease, height 1200ms ease, x 1200ms ease, y 1200ms ease' }}
                      />
                    ) : (
                      <circle
                        r={radius}
                        fill={color}
                        fillOpacity={isFocused ? 1 : 0.88}
                        stroke="#0b0c0f"
                        strokeWidth={2}
                        filter={isFocused ? 'url(#node-glow)' : undefined}
                        style={{ transition: 'r 150ms ease' }}
                      />
                    )}
                    <text
                      y={radius + 16}
                      textAnchor="middle"
                      className="pointer-events-none select-none text-[11px]"
                      fill={isFocused ? '#e4e4e7' : isBase ? '#c4c4cc' : '#9b9ba5'}
                      fontWeight={isBase ? 600 : 400}
                    >
                      {n.title.length > 22 ? `${n.title.slice(0, 22)}…` : n.title}
                    </text>
                  </g>
                );
              })}
            </g>
          </svg>
        )}

        <div className="pointer-events-none absolute bottom-3 left-3 max-w-md text-xs text-zinc-600">
          Scroll to zoom · drag to pan · drag a node to move it · double-click a note to open · Shift+drag onto
          another note or a base to link (target highlights green) · right-click for more options
        </div>
        <div className="absolute right-3 top-3 flex items-center gap-2">
          <span className="rounded-md bg-ink-900/80 px-2 py-1 text-xs text-zinc-500">
            {Math.round(transform.k * 100)}%
          </span>
          <button
            onClick={resetView}
            title="Reset view"
            className="rounded-md bg-ink-900/80 p-1.5 text-zinc-500 hover:text-accent"
          >
            <Maximize2 size={14} />
          </button>
        </div>
      </div>

      <aside className="w-64 shrink-0 overflow-y-auto border-l border-ink-800 bg-ink-900 px-4 py-5">
        <h3 className="mb-4 text-xs font-semibold uppercase tracking-wide text-zinc-500">Filter & color</h3>
        <section className="mb-5">
          <SectionLabel icon={<Filter size={11} />}>Filters</SectionLabel>
          <label className="flex items-center gap-2 text-xs text-zinc-400">
            <input
              type="checkbox"
              checked={onlyConnected}
              onChange={(e) => setOnlyConnected(e.target.checked)}
              className="accent-accent"
            />
            Hide notes with no links
          </label>
        </section>
        <section className="mb-5">
          <SectionLabel icon={<Palette size={11} />}>Coloring</SectionLabel>
          <div className="flex items-start gap-1.5 rounded-md border border-ink-800 bg-ink-950 px-2.5 py-2 text-xs text-zinc-500">
            <Network size={12} className="mt-0.5 shrink-0" />
            <span>
              Linked notes always share a color automatically. Linking a note to a Base (the diamond shapes)
              colors it to match -- change the base&apos;s color any time and every note linked to it follows.
            </span>
          </div>
        </section>
        <section>
          <SectionLabel icon={<TagIcon size={11} />}>Tags</SectionLabel>
          {tags.length === 0 ? (
            <div className="rounded-md border border-dashed border-ink-700 px-2.5 py-3 text-xs text-zinc-600">
              No tags yet. Open a note and add one from the right panel to start filtering and coloring by topic.
            </div>
          ) : (
            <div className="space-y-1">
              {tags.map((t) => {
                const active = selectedTagIds.has(t.id);
                const count = tagNoteCounts.get(t.id) ?? 0;
                return (
                  <div
                    key={t.id}
                    className={`flex items-center gap-2 rounded-md px-2 py-1.5 text-xs transition ${
                      active ? 'bg-ink-700 text-zinc-100' : 'text-zinc-400 hover:bg-ink-800'
                    }`}
                  >
                    <button onClick={() => toggleTagFilter(t.id)} className="flex min-w-0 flex-1 items-center gap-1.5 text-left">
                      <span className="truncate">{t.name}</span>
                      <span className="shrink-0 text-zinc-600">{count}</span>
                    </button>
                    <input
                      type="color"
                      value={t.color ?? '#7c6cf6'}
                      onChange={(e) => onTagColorChange(t.id, e.target.value)}
                      title="Change tag color"
                      className="h-3.5 w-3.5 shrink-0 cursor-pointer rounded-full border-0 bg-transparent p-0 [&::-webkit-color-swatch]:rounded-full [&::-webkit-color-swatch]:border-0 [&::-webkit-color-swatch-wrapper]:rounded-full [&::-webkit-color-swatch-wrapper]:p-0"
                    />
                  </div>
                );
              })}
            </div>
          )}
        </section>

        {filtersActive && (
          <button onClick={clearFilters} className="mt-4 text-xs text-accent hover:underline">
            Clear filters
          </button>
        )}
      </aside>

      {noteContextMenu && (
        <NoteContextMenu
          note={noteContextMenu.note}
          x={noteContextMenu.x}
          y={noteContextMenu.y}
          onClose={() => setNoteContextMenu(null)}
          onOpen={onOpenNote}
          onTogglePin={onTogglePin}
          onRename={onRename}
          onSetColor={onSetColor}
          onDelete={onDelete}
        />
      )}

      {baseContextMenu && (
        <BaseContextMenu
          base={baseContextMenu.base}
          x={baseContextMenu.x}
          y={baseContextMenu.y}
          onClose={() => setBaseContextMenu(null)}
          onRename={onRenameBase}
          onSetColor={onSetBaseColor}
          onDelete={onDeleteBase}
        />
      )}
    </div>
  );
}

function SectionLabel({ icon, children }: { icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <h4 className="mb-2 flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
      {icon}
      {children}
    </h4>
  );
}

import type { Base, BaseNoteAssignment, Link, NoteSummary } from './types';

export const CLUSTER_PALETTE = [
  '#f97316',
  '#22c55e',
  '#3b82f6',
  '#ec4899',
  '#eab308',
  '#06b6d4',
  '#a855f7',
  '#ef4444',
  '#14b8a6',
  '#84cc16',
  '#6366f1',
  '#f43f5e',
];

export function computeAutoColors(
  notes: NoteSummary[],
  links: Link[],
  bases: Base[],
  baseNoteAssignments: BaseNoteAssignment[],
): Record<string, string> {
  const noteIds = new Set(notes.map((n) => n.id));
  const baseIds = new Set(bases.map((b) => b.id));
  const baseColorById = new Map(bases.map((b) => [b.id, b.color]));
  const allIds = new Set<string>([...noteIds, ...baseIds]);

  const adjacency = new Map<string, Set<string>>();
  for (const id of allIds) adjacency.set(id, new Set());

  for (const link of links) {
    if (!link.target_note_id) continue;
    if (!noteIds.has(link.source_note_id) || !noteIds.has(link.target_note_id)) continue;
    if (link.source_note_id === link.target_note_id) continue;
    adjacency.get(link.source_note_id)!.add(link.target_note_id);
    adjacency.get(link.target_note_id)!.add(link.source_note_id);
  }
  for (const a of baseNoteAssignments) {
    if (!noteIds.has(a.note_id) || !baseIds.has(a.base_id)) continue;
    adjacency.get(a.note_id)!.add(a.base_id);
    adjacency.get(a.base_id)!.add(a.note_id);
  }

  const visited = new Set<string>();
  const components: string[][] = [];

  for (const id of allIds) {
    if (visited.has(id)) continue;
    const component: string[] = [];
    const stack = [id];
    visited.add(id);
    while (stack.length > 0) {
      const current = stack.pop()!;
      component.push(current);
      for (const neighbor of adjacency.get(current) ?? []) {
        if (!visited.has(neighbor)) {
          visited.add(neighbor);
          stack.push(neighbor);
        }
      }
    }
    components.push(component);
  }

  const sortedComponents = components.sort((a, b) => {
    const aMin = a.reduce((min, id) => (id < min ? id : min));
    const bMin = b.reduce((min, id) => (id < min ? id : min));
    return aMin < bMin ? -1 : aMin > bMin ? 1 : 0;
  });

  const colorMap: Record<string, string> = {};
  let paletteIndex = 0;

  for (const component of sortedComponents) {
    const basesInComponent = component
      .filter((id) => baseIds.has(id))
      .sort();

    let color: string;
    if (basesInComponent.length > 0) {
      color = baseColorById.get(basesInComponent[0]!)!;
    } else {
      color = CLUSTER_PALETTE[paletteIndex % CLUSTER_PALETTE.length] ?? CLUSTER_PALETTE[0]!;
      paletteIndex++;
    }

    for (const id of component) {
      if (noteIds.has(id)) colorMap[id] = color;
    }
  }

  return colorMap;
}

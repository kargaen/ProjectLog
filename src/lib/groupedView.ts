// Two-level grouped view model for the project list (EPIC-008).
// Level 1 mixes group entries and ungrouped project entries; groups carry their members.
export type GroupedViewEntry =
  | { kind: "group"; name: string; projects: string[] }
  | { kind: "project"; name: string };

export type GroupedView = GroupedViewEntry[];

export type GroupedViewMode = "manual" | "alphabetical" | "recent";

export function buildGroupedView(
  orderedProjects: string[],
  groups: Record<string, string>,
  mode: GroupedViewMode,
  recentUsage: Record<string, number>,
): GroupedView {
  const members = new Map<string, string[]>();
  const entries: GroupedView = [];

  for (const project of orderedProjects) {
    const groupName = groups[project];
    if (groupName === undefined) {
      entries.push({ kind: "project", name: project });
      continue;
    }
    const existing = members.get(groupName);
    if (existing) {
      existing.push(project);
    } else {
      const projects = [project];
      members.set(groupName, projects);
      // The group takes level-1 position at its first member; manual keeps that order.
      entries.push({ kind: "group", name: groupName, projects });
    }
  }

  if (mode === "alphabetical") {
    entries.sort((a, b) => a.name.localeCompare(b.name));
    for (const projects of members.values()) {
      projects.sort((a, b) => a.localeCompare(b));
    }
  }

  if (mode === "recent") {
    // A group's recency is the most recent among its members (T_group = max).
    const recency = (entry: GroupedViewEntry): number =>
      entry.kind === "project"
        ? (recentUsage[entry.name] ?? 0)
        : Math.max(...entry.projects.map((p) => recentUsage[p] ?? 0));
    entries.sort((a, b) => recency(b) - recency(a) || a.name.localeCompare(b.name));
    for (const projects of members.values()) {
      projects.sort(
        (a, b) => (recentUsage[b] ?? 0) - (recentUsage[a] ?? 0) || a.localeCompare(b),
      );
    }
  }

  return entries;
}

import { expect, test } from "@playwright/test";
import { buildGroupedView } from "../../src/lib/groupedView";

// Shared fixture: two named groups plus two ungrouped projects.
const GROUPS: Record<string, string> = {
  Mango: "Work",
  Echo: "Work",
  Kilo: "Baking",
};

test.describe("buildGroupedView — alphabetical (flow 6)", () => {
  const ordered = ["Zeta", "Mango", "Echo", "Kilo", "Alpha"];

  test("level one mixes group names and ungrouped project names, A-Z together", () => {
    const view = buildGroupedView(ordered, GROUPS, "alphabetical", {});
    expect(
      view.map((entry) => ({ kind: entry.kind, name: entry.name })),
    ).toEqual([
      { kind: "project", name: "Alpha" },
      { kind: "group", name: "Baking" },
      { kind: "group", name: "Work" },
      { kind: "project", name: "Zeta" },
    ]);
  });

  test("level two orders each group's members A-Z", () => {
    const view = buildGroupedView(ordered, GROUPS, "alphabetical", {});
    const work = view.find((entry) => entry.kind === "group" && entry.name === "Work");
    expect(work).toEqual({ kind: "group", name: "Work", projects: ["Echo", "Mango"] });
    const baking = view.find((entry) => entry.kind === "group" && entry.name === "Baking");
    expect(baking).toEqual({ kind: "group", name: "Baking", projects: ["Kilo"] });
  });
});

test.describe("buildGroupedView — recent (flow 7)", () => {
  const ordered = ["Zeta", "Mango", "Echo", "Kilo", "Alpha"];
  // T_group = max(member timestamps): Work = 900 (Echo), Baking = 200 (Kilo).
  const usage: Record<string, number> = {
    Alpha: 100,
    Zeta: 500,
    Mango: 300,
    Echo: 900,
    Kilo: 200,
  };

  test("level one orders groups by their most recent member, mixed with ungrouped, newest first", () => {
    const view = buildGroupedView(ordered, GROUPS, "recent", usage);
    expect(
      view.map((entry) => ({ kind: entry.kind, name: entry.name })),
    ).toEqual([
      { kind: "group", name: "Work" },
      { kind: "project", name: "Zeta" },
      { kind: "group", name: "Baking" },
      { kind: "project", name: "Alpha" },
    ]);
  });

  test("level two orders each group's members by recency, newest first", () => {
    const view = buildGroupedView(ordered, GROUPS, "recent", usage);
    const work = view.find((entry) => entry.kind === "group" && entry.name === "Work");
    expect(work).toEqual({ kind: "group", name: "Work", projects: ["Echo", "Mango"] });
  });
});

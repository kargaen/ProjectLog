import { expect, test } from "@playwright/test";
import {
  buildTimesheetDisplayRows,
  findRoundingScale,
  roundToHourStep,
} from "../../src/lib/timesheet-preview";
import type { TimesheetPreviewSheet } from "../../src/models/types";

function row(
  label: string,
  values: number[],
  flags: { is_comment?: boolean; is_total?: boolean } = {},
) {
  return {
    label,
    values,
    total: values.reduce((sum, value) => sum + value, 0),
    is_comment: flags.is_comment ?? false,
    is_total: flags.is_total ?? false,
  };
}

function sheetOf(rows: ReturnType<typeof row>[]): TimesheetPreviewSheet {
  return { name: "2026-18", columns: ["Mon", "Tue", "Wed"], rows };
}

function countedCells(rows: { values: number[]; is_comment: boolean; is_total: boolean }[]) {
  return rows
    .filter((entry) => !entry.is_comment && !entry.is_total)
    .flatMap((entry) => entry.values);
}

function sum(values: number[]) {
  return values.reduce((total, value) => total + value, 0);
}

function isHalfHour(value: number) {
  return Math.abs(value / 0.5 - Math.round(value / 0.5)) < 0.000001;
}

test.describe("findRoundingScale", () => {
  test("stays at 1 when the cells already add up to the target", () => {
    expect(findRoundingScale([2.4, 1.1, 0.7, 3.9], 8)).toBe(1);
  });

  test("scales up when rounding each cell alone falls short", () => {
    // 1.2 + 1.2 = 2.4, which rounds to 2.5; rounding each cell alone gives 2.0.
    const values = [1.2, 1.2];
    const scale = findRoundingScale(values, 2.5);

    expect(scale).toBeGreaterThan(1);
    expect(scale).toBeLessThanOrEqual(2);
  });

  test("scales down when rounding each cell alone overshoots", () => {
    // 0.3 x 4 = 1.2, which rounds to 1.0; rounding each cell alone gives 2.0.
    const values = [0.3, 0.3, 0.3, 0.3];
    const scale = findRoundingScale(values, 1);

    expect(scale).toBeLessThan(1);
    expect(scale).toBeGreaterThan(0);
  });

  test("separates cells holding identical hours instead of moving them together", () => {
    const values = [0.3, 0.3, 0.3, 0.3];
    const scale = findRoundingScale(values, 1);
    const rounded = values.map(
      (value, index) => roundToHourStep(value * scale * (1 + index * 1e-9)),
    );

    // Without a tie-break every cell would cross at once: 0 or 2.0, never 1.0.
    expect(sum(rounded)).toBeCloseTo(1, 6);
  });
});

test.describe("buildTimesheetDisplayRows — rounded cells add up to the rounded total", () => {
  const cases: Record<string, TimesheetPreviewSheet> = {
    "row totals that each round down": sheetOf([
      row("Alpha", [1.2, 0, 0]),
      row("Beta", [0, 1.2, 0]),
      row("Total", [1.2, 1.2, 0], { is_total: true }),
    ]),
    "identical small cells": sheetOf([
      row("Alpha", [0.3, 0.3, 0.3]),
      row("Beta", [0.3, 0.3, 0.3]),
      row("Total", [0.6, 0.6, 0.6], { is_total: true }),
    ]),
    "comment rows excluded from the total": sheetOf([
      row("Alpha", [2.4, 1.1, 0]),
      row("  - Prep", [1.4, 0, 0], { is_comment: true }),
      row("  - Build", [1.0, 1.1, 0], { is_comment: true }),
      row("Beta", [0, 0.7, 3.9]),
      row("Total", [2.4, 1.8, 3.9], { is_total: true }),
    ]),
    "cells sitting exactly on the quarter hour": sheetOf([
      row("Alpha", [1.75, 0.25, 2.25]),
      row("Beta", [0.75, 1.25, 0.25]),
      row("Gamma", [3.25, 0, 1.75]),
      row("Total", [5.75, 1.5, 4.25], { is_total: true }),
    ]),
    "whole hours are left alone": sheetOf([
      row("Alpha", [1, 2, 0]),
      row("Beta", [0, 3, 4]),
      row("Total", [1, 5, 4], { is_total: true }),
    ]),
  };

  for (const [name, sheet] of Object.entries(cases)) {
    test(name, () => {
      const rows = buildTimesheetDisplayRows(sheet, true);
      const totalRow = rows.find((entry) => entry.is_total)!;

      // The headline number is the raw total rounded, and the cells add up to it.
      expect(totalRow.total).toBeCloseTo(
        roundToHourStep(sum(countedCells(sheet.rows))),
        6,
      );
      expect(sum(countedCells(rows))).toBeCloseTo(totalRow.total, 6);

      for (const entry of rows) {
        for (const value of entry.values) {
          expect(isHalfHour(value)).toBe(true);
        }
        expect(entry.total).toBeCloseTo(sum(entry.values), 6);
      }
    });
  }

  test("a day with no hours never gains any", () => {
    for (const sheet of Object.values(cases)) {
      const rows = buildTimesheetDisplayRows(sheet, true);

      rows.forEach((entry, rowIndex) => {
        entry.values.forEach((value, columnIndex) => {
          if (sheet.rows[rowIndex].values[columnIndex] === 0) {
            expect(value).toBe(0);
          }
        });
      });
    }
  });

  test("whole hours survive rounding unchanged", () => {
    const rows = buildTimesheetDisplayRows(cases["whole hours are left alone"], true);

    expect(rows[0].values).toEqual([1, 2, 0]);
    expect(rows[1].values).toEqual([0, 3, 4]);
  });

  test("rounding disabled leaves the sheet exactly as received", () => {
    const sheet = cases["cells sitting exactly on the quarter hour"];
    const rows = buildTimesheetDisplayRows(sheet, false);

    expect(rows.map((entry) => entry.values)).toEqual(
      sheet.rows.map((entry) => entry.values),
    );
  });
});

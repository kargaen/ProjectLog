import type { TimesheetPreviewRow, TimesheetPreviewSheet } from "./types";

export type TimesheetDisplayRow = TimesheetPreviewRow & {
  band_index: number;
};

export function roundHalfPreservingSum(values: number[]) {
  const step = 0.5;
  const floors = values.map((value) => Math.floor(value / step) * step);
  const roundedTotal = Math.round(values.reduce((sum, value) => sum + value, 0) / step) * step;
  const currentTotal = floors.reduce((sum, value) => sum + value, 0);
  let increments = Math.round((roundedTotal - currentTotal) / step);
  const ranked = values
    .map((value, index) => ({ index, remainder: value - floors[index] }))
    .sort((a, b) => b.remainder - a.remainder || a.index - b.index);
  const result = [...floors];

  for (let i = 0; i < ranked.length && increments > 0; i += 1) {
    result[ranked[i].index] += step;
    increments -= 1;
  }

  return result;
}

export function buildTimesheetDisplayRows(
  sheet: TimesheetPreviewSheet | null,
  roundingEnabled: boolean
) {
  if (!sheet) return [];

  let currentBand = -1;
  const rows: TimesheetDisplayRow[] = sheet.rows.map((row) => {
    if (!row.is_comment && !row.is_total) {
      currentBand += 1;
    }

    const bandIndex = row.is_total ? -1 : Math.max(currentBand, 0);
    if (!roundingEnabled || row.is_total) {
      return {
        ...row,
        band_index: bandIndex,
      };
    }

    const roundedValues = roundHalfPreservingSum(row.values);
    return {
      ...row,
      values: roundedValues,
      total: roundedValues.reduce((sum, value) => sum + value, 0),
      band_index: bandIndex,
    };
  });

  if (!roundingEnabled) {
    return rows;
  }

  const totalIndex = rows.findIndex((row) => row.is_total);
  if (totalIndex === -1) {
    return rows;
  }

  const valueCount = rows[totalIndex].values.length;
  const columnTotals = new Array<number>(valueCount).fill(0);
  for (const row of rows) {
    if (row.is_total) continue;
    row.values.forEach((value, index) => {
      columnTotals[index] += value;
    });
  }

  rows[totalIndex] = {
    ...rows[totalIndex],
    values: columnTotals,
    total: columnTotals.reduce((sum, value) => sum + value, 0),
  };

  return rows;
}

export function formatPreviewHours(value: number, roundingEnabled: boolean) {
  return roundingEnabled ? value.toFixed(1) : value.toFixed(2);
}

export function formatGeneratedTimestamp(timestampMs: number) {
  const date = new Date(timestampMs);
  const pad = (value: number) => String(value).padStart(2, "0");

  return [
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
    `${pad(date.getHours())}:${pad(date.getMinutes())}`,
  ].join(" ");
}

function pluralize(value: number, unit: string) {
  return `${value} ${unit}${value === 1 ? "" : "s"} ago`;
}

export function formatGeneratedAge(timestampMs: number, nowMs: number) {
  const elapsedSeconds = Math.max(0, Math.floor((nowMs - timestampMs) / 1000));
  if (elapsedSeconds < 60) {
    return pluralize(elapsedSeconds, "second");
  }

  const elapsedMinutes = Math.floor(elapsedSeconds / 60);
  if (elapsedMinutes < 60) {
    return pluralize(elapsedMinutes, "minute");
  }

  const elapsedHours = Math.floor(elapsedMinutes / 60);
  if (elapsedHours < 24) {
    return pluralize(elapsedHours, "hour");
  }

  return pluralize(Math.floor(elapsedHours / 24), "day");
}

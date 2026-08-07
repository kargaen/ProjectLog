import type { TimesheetPreviewRow, TimesheetPreviewSheet } from "../models/types";

export type TimesheetDisplayRow = TimesheetPreviewRow & {
  band_index: number;
};

const ZERO_EPSILON = 0.000001;

const HOUR_STEP = 0.5;
// Lower bound rounds every cell down to zero, upper bound doubles every cell: any
// reachable target sum lies between the two. Midpoint of the first bisection is
// exactly 1, so an already-consistent sheet costs a single iteration.
const SCALE_LOWER_BOUND = 0;
const SCALE_UPPER_BOUND = 2;
const SCALE_BISECT_ITERATIONS = 40;
const SUM_EPSILON = 0.000001;
// Cells holding the same hours would otherwise cross the half-hour boundary at the
// same scale, moving the sum by a whole group at a time and stepping over the
// target. Weighting each cell by its position separates those crossings, so the
// sum climbs half an hour at a time and the bisect can land on the target. The
// weight is far too small to move a cell across a boundary on its own.
const TIE_BREAK_WEIGHT = 0.000000001;

function isZeroHours(value: number) {
  return Math.abs(value) < ZERO_EPSILON;
}

export function roundToHourStep(value: number) {
  return Math.round(value / HOUR_STEP) * HOUR_STEP;
}

function roundCell(value: number, scale: number, cellIndex: number) {
  return roundToHourStep(value * scale * (1 + cellIndex * TIE_BREAK_WEIGHT));
}

function scaledRoundedSum(values: number[], scale: number) {
  return values.reduce(
    (sum, value, index) => sum + roundCell(value, scale, index),
    0
  );
}

/**
 * Bisects a scale factor until the cells, scaled and then rounded to whole half
 * hours, add up to `target`. The sum is non-decreasing in the scale, so halving
 * the bracket each step converges on it. A target no scale can reach falls back
 * to the scale that came closest.
 */
export function findRoundingScale(values: number[], target: number) {
  let low = SCALE_LOWER_BOUND;
  let high = SCALE_UPPER_BOUND;
  let bestScale = SCALE_UPPER_BOUND;
  let bestDistance = Infinity;

  for (let i = 0; i < SCALE_BISECT_ITERATIONS; i += 1) {
    const scale = (low + high) / 2;
    const sum = scaledRoundedSum(values, scale);
    const distance = Math.abs(sum - target);

    if (distance < bestDistance) {
      bestDistance = distance;
      bestScale = scale;
    }

    if (distance < SUM_EPSILON) {
      return scale;
    }

    if (sum < target) {
      low = scale;
    } else {
      high = scale;
    }
  }

  return bestScale;
}

/**
 * One scale factor per sheet, chosen so the sheet's rounded cells add up to the
 * sheet's rounded total. Comment rows break a project's own hours down further,
 * so they stay out of the target — counting them would double the sheet.
 */
function sheetRoundingScale(sheet: TimesheetPreviewSheet) {
  const countedValues = sheet.rows
    .filter((row) => !row.is_comment && !row.is_total)
    .flatMap((row) => row.values);
  const targetTotal = roundToHourStep(
    countedValues.reduce((sum, value) => sum + value, 0)
  );

  return findRoundingScale(countedValues, targetTotal);
}

export function buildTimesheetDisplayRows(
  sheet: TimesheetPreviewSheet | null,
  roundingEnabled: boolean
) {
  if (!sheet) return [];

  const scale = roundingEnabled ? sheetRoundingScale(sheet) : 1;

  let currentBand = -1;
  // Counts the cells the scale was fitted against, in the order it saw them, so
  // each one keeps the tie-break weight the bisect assigned it.
  let countedCellIndex = 0;
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

    const roundedValues = row.values.map((value) =>
      row.is_comment
        ? roundToHourStep(value * scale)
        : roundCell(value, scale, countedCellIndex++)
    );
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
    if (row.is_total || row.is_comment) continue;
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
  if (isZeroHours(value)) {
    return "-";
  }

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

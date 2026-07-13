import { expect, test } from "@playwright/test";
import { clampMenuPosition } from "../../src/lib/menuPosition";

const MENU = { width: 170, height: 200 };
const VIEWPORT = { width: 430, height: 650 };

test.describe("clampMenuPosition", () => {
  test("returns the click point unchanged when the menu already fits", () => {
    expect(clampMenuPosition({ x: 40, y: 60 }, MENU, VIEWPORT)).toEqual({ x: 40, y: 60 });
  });

  test("pulls the menu left when it would overflow the right edge", () => {
    const pos = clampMenuPosition({ x: 400, y: 60 }, MENU, VIEWPORT);
    expect(pos.x).toBe(VIEWPORT.width - MENU.width); // 260
    expect(pos.y).toBe(60);
    expect(pos.x + MENU.width).toBeLessThanOrEqual(VIEWPORT.width);
  });

  test("pulls the menu up when it would overflow the bottom edge", () => {
    const pos = clampMenuPosition({ x: 40, y: 600 }, MENU, VIEWPORT);
    expect(pos.x).toBe(40);
    expect(pos.y).toBe(VIEWPORT.height - MENU.height); // 450
    expect(pos.y + MENU.height).toBeLessThanOrEqual(VIEWPORT.height);
  });

  test("clamps both axes and never goes negative", () => {
    const pos = clampMenuPosition({ x: 999, y: 999 }, MENU, VIEWPORT);
    expect(pos.x).toBe(VIEWPORT.width - MENU.width);
    expect(pos.y).toBe(VIEWPORT.height - MENU.height);

    // Menu larger than the viewport still pins to the top-left corner.
    const tiny = clampMenuPosition({ x: 10, y: 10 }, { width: 500, height: 800 }, VIEWPORT);
    expect(tiny).toEqual({ x: 0, y: 0 });
  });
});

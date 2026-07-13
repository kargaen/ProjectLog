export type Point = { x: number; y: number };
export type Size = { width: number; height: number };

// Return the top-left at which a menu of `menu` size, opened at `click`, sits fully inside
// a viewport of `viewport` size. Overflow past the right/bottom edge pulls the menu back
// flush with that edge; a menu larger than the viewport pins to the top-left corner.
export function clampMenuPosition(click: Point, menu: Size, viewport: Size): Point {
  return {
    x: Math.max(0, Math.min(click.x, viewport.width - menu.width)),
    y: Math.max(0, Math.min(click.y, viewport.height - menu.height)),
  };
}

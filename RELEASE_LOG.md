Refactor timesheet preview and lighten dedicated window startup

- Split the dedicated timesheet preview into its own frontend component and bootstrap flow so preview loading no longer piggybacks on the QuickPanel lifecycle.
- Refactored the Rust timesheet pipeline so preview and Excel export share one report model, reducing duplication and drift between data and view behavior.
- Added banded preview rows, darker header and total styling, row-and-column crosshair hover highlighting, and a live generated timestamp with an `Update now` refresh action.

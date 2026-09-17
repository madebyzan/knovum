const IMPORT_GAP = 24;

export function layoutImportGrid(
  sizes: readonly { width: number; height: number }[],
  origin: { x: number; y: number },
) {
  if (!sizes.length) return [];
  const columns = Math.min(3, Math.ceil(Math.sqrt(sizes.length)));
  const columnWidths = Array.from({ length: columns }, (_columnValue, column) =>
    Math.max(...sizes.filter((_size, index) => index % columns === column).map(({ width }) => width)),
  );
  const rows = Math.ceil(sizes.length / columns);
  const rowHeights = Array.from({ length: rows }, (_rowValue, row) =>
    Math.max(...sizes.slice(row * columns, (row + 1) * columns).map(({ height }) => height)),
  );
  return sizes.map((_size, index) => {
    const column = index % columns;
    const row = Math.floor(index / columns);
    return {
      x: origin.x + columnWidths.slice(0, column).reduce((sum, width) => sum + width + IMPORT_GAP, 0),
      y: origin.y + rowHeights.slice(0, row).reduce((sum, height) => sum + height + IMPORT_GAP, 0),
    };
  });
}

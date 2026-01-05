import type { CellPosition, RowHeightValue } from '$lib/components/data-grid/data-logic/types';
import { ROW_HEIGHT_VALUES, ROW_LINE_COUNTS } from '$lib/components/data-grid/data-logic/config';
import type { Column } from '@tanstack/table-core';

/**
 * Creates a unique cell key from row index and column id
 */
export function getCellKey(rowIndex: number, columnId: string): string {
	return `${rowIndex}:${columnId}`;
}

/**
 * Parses a cell key back into row index and column id
 */
export function parseCellKey(cellKey: string): CellPosition {
	const parts = cellKey.split(':');
	const rowIndexStr = parts[0];
	const columnId = parts[1];
	if (rowIndexStr && columnId) {
		const rowIndex = parseInt(rowIndexStr, 10);
		if (!Number.isNaN(rowIndex)) {
			return { rowIndex, columnId };
		}
	}
	return { rowIndex: 0, columnId: '' };
}

/**
 * Gets the pixel height for a row height value
 */
export function getRowHeightValue(rowHeight: RowHeightValue): number {
	return ROW_HEIGHT_VALUES[rowHeight];
}

/**
 * Gets the line count for a row height value
 */
export function getLineCount(rowHeight: RowHeightValue): number {
	return ROW_LINE_COUNTS[rowHeight];
}

/**
 * Gets common pinning styles for a column (port of TableCN's getCommonPinningStyles)
 */
export function getCommonPinningStyles<TData>(params: {
	column?: Column<TData, unknown>;
	withBorder?: boolean;
}): Record<string, string | number | undefined> {
	const { column, withBorder = false } = params;

	// Return default styles if column is undefined
	if (!column) {
		return {
			position: 'relative',
			background: 'var(--background)',
			zIndex: undefined
		};
	}

	// Wrap in try-catch to handle SSR edge cases where TanStack internal state may not be ready
	try {
		const isPinned = column.getIsPinned();
		const isLastLeftPinnedColumn = isPinned === 'left' && column.getIsLastColumn('left');
		const isFirstRightPinnedColumn = isPinned === 'right' && column.getIsFirstColumn('right');

		return {
			boxShadow: withBorder
				? isLastLeftPinnedColumn
					? '-4px 0 4px -4px var(--border) inset'
					: isFirstRightPinnedColumn
						? '4px 0 4px -4px var(--border) inset'
						: undefined
				: undefined,
			left: isPinned === 'left' ? `${column.getStart('left')}px` : undefined,
			right: isPinned === 'right' ? `${column.getAfter('right')}px` : undefined,
			opacity: isPinned ? 0.97 : 1,
			position: isPinned ? 'sticky' : 'relative',
			background: isPinned ? 'var(--background)' : 'var(--background)',
			width: column.getSize(),
			zIndex: isPinned ? 1 : undefined
		};
	} catch {
		// Return default styles if column methods fail (e.g., during SSR)
		return {
			position: 'relative',
			background: 'var(--background)',
			zIndex: undefined
		};
	}
}
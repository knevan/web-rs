// Data Grid Types for TableCN-Svelte
// Exact port of TableCN React types for Svelte 5

import type {
	ColumnDef,
	Table,
	Row,
	Cell,
	Column,
	RowData
} from '@tanstack/table-core';
import type { SvelteSet, SvelteMap } from 'svelte/reactivity';
import type { Snippet, Component } from 'svelte';

// ============================================
// Base Types
// ============================================

export interface Option {
	label: string;
	value: string;
}

export type RowHeightValue = 'short' | 'medium' | 'tall' | 'extra-tall';

export interface CellSelectOption {
	label: string;
	value: string;
	icon?: Component;
	count?: number;
}

// ============================================
// Header Types
// ============================================

export interface HeaderOptions {
	showTooltip?: boolean;
	showIcon?: boolean;
	showDropdown?: boolean;
	showSortIndicator?: boolean;
}

// ============================================
// Cell Types
// ============================================

export type CellOpts =
	| { variant: 'short-text' }
	| { variant: 'long-text' }
	| { variant: 'number'; min?: number; max?: number; step?: number }
	| { variant: 'select'; options: CellSelectOption[] }
	| { variant: 'multi-select'; options: CellSelectOption[] }
	| { variant: 'checkbox' }
	| { variant: 'date' }
	| { variant: 'url' }
	| { variant: 'row-select' }
	| { variant: 'actions' }
	| {
	variant: 'file';
	maxFileSize?: number;
	maxFiles?: number;
	accept?: string;
	multiple?: boolean;
}
	| {
	variant: 'badge';
	options?: CellSelectOption[];
	variantMap?: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'>;
	classMap?: Record<string, string>;
};

export interface UpdateCell {
	rowIndex: number;
	columnId: string;
	value: unknown;
}

// ============================================
// Position & Selection Types
// ============================================

export interface CellPosition {
	rowIndex: number;
	columnId: string;
}

export interface CellRange {
	start: CellPosition;
	end: CellPosition;
}

export interface SelectionState {
	selectedCells: Set<string>;
	selectionRange: CellRange | null;
	isSelecting: boolean;
}

// ============================================
// Context Menu Types
// ============================================

export interface ContextMenuState {
	open: boolean;
	x: number;
	y: number;
}

// ============================================
// Paste Dialog Types
// ============================================

export interface PasteDialogState {
	open: boolean;
	rowsNeeded: number;
	clipboardText: string;
}

// ============================================
// Navigation Types
// ============================================

export type NavigationDirection =
	| 'up'
	| 'down'
	| 'left'
	| 'right'
	| 'home'
	| 'end'
	| 'ctrl+home'
	| 'ctrl+end'
	| 'pageup'
	| 'pagedown';

// ============================================
// Search Types
// ============================================

// Type alias for search match - same as CellPosition
export type SearchMatch = CellPosition;

// Data-only search state (used by stores)
export interface SearchStateData {
	searchOpen: boolean;
	searchQuery: string;
	searchMatches: SearchMatch[];
	matchIndex: number;
}

// Full search state with callbacks (used by components)
export interface SearchState extends SearchStateData {
	onSearchOpenChange: (open: boolean) => void;
	onSearchQueryChange: (query: string) => void;
	onSearch: (query: string) => void;
	onNavigateToNextMatch: () => void;
	onNavigateToPrevMatch: () => void;
}

// ============================================
// Cell Variant Props
// ============================================

export interface CellVariantProps<TData> {
	cell: Cell<TData, unknown>;
	table: Table<TData>;
	rowIndex: number;
	columnId: string;
	isEditing: boolean;
	isFocused: boolean;
	isSelected: boolean;
	readOnly?: boolean;
	/** Centralized cell value with fine-grained reactivity from SvelteMap */
	cellValue: unknown;
}

// ============================================
// File Cell Types
// ============================================

export interface FileCellData {
	id: string;
	name: string;
	size: number;
	type: string;
	url?: string;
}

// ============================================
// Filter Types
// ============================================

export type FilterVariant =
	| 'text'
	| 'number'
	| 'range'
	| 'date'
	| 'dateRange'
	| 'boolean'
	| 'select'
	| 'multiSelect';

export type TextFilterOperator =
	| 'contains'
	| 'notContains'
	| 'equals'
	| 'notEquals'
	| 'startsWith'
	| 'endsWith'
	| 'isEmpty'
	| 'isNotEmpty';

export type NumberFilterOperator =
	| 'equals'
	| 'notEquals'
	| 'lessThan'
	| 'lessThanOrEqual'
	| 'greaterThan'
	| 'greaterThanOrEqual'
	| 'between'
	| 'isEmpty'
	| 'isNotEmpty';

export type DateFilterOperator =
	| 'equals'
	| 'notEquals'
	| 'before'
	| 'after'
	| 'onOrBefore'
	| 'onOrAfter'
	| 'between'
	| 'isEmpty'
	| 'isNotEmpty';

export type SelectFilterOperator =
	| 'is'
	| 'isNot'
	| 'isAnyOf'
	| 'isNoneOf'
	| 'isEmpty'
	| 'isNotEmpty';

export type BooleanFilterOperator = 'isTrue' | 'isFalse';

export type FilterOperator =
	| TextFilterOperator
	| NumberFilterOperator
	| DateFilterOperator
	| SelectFilterOperator
	| BooleanFilterOperator;

export interface FilterValue {
	operator: FilterOperator;
	value?: string | number | string[];
	value2?: string | number;
}

// ============================================
// TanStack Table Meta Extension
// ============================================

declare module '@tanstack/table-core' {
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	interface ColumnMeta<TData extends RowData, TValue> {
		// Generic properties
		label?: string;

		// Header configuration
		headerOptions?: HeaderOptions;

		// Cell rendering
		cell?: CellOpts;

		// Filtering properties
		placeholder?: string;
		variant?: FilterVariant;
		options?: { label: string; value: string }[];
		range?: [number, number];
		unit?: string;

		// Styling properties
		headerClass?: string;
		cellClass?: string;
	}

	interface TableMeta<TData extends RowData> {
		dataGridRef?: HTMLElement | null;
		cellMapRef?: Map<string, HTMLDivElement>;
		focusedCell?: CellPosition | null;
		editingCell?: CellPosition | null;
		selectionState?: SelectionState;
		searchOpen?: boolean;
		readOnly?: boolean;
		getIsCellSelected?: (rowIndex: number, columnId: string) => boolean;
		// SvelteMap for fine-grained cell value reactivity - cells access map.get(key) in $derived
		cellValueMap?: SvelteMap<string, unknown>;
		// SvelteSet for fine-grained cell selection reactivity
		selectedCellsSet?: SvelteSet<string>;
		// Version counter to force cell re-renders when selection changes
		selectionVersion?: number;
		getIsSearchMatch?: (rowIndex: number, columnId: string) => boolean;
		getIsActiveSearchMatch?: (rowIndex: number, columnId: string) => boolean;
		// SvelteSet for fine-grained reactive search match lookups
		searchMatchSet?: SvelteSet<string>;
		activeSearchMatch?: CellPosition | null;
		rowHeight?: RowHeightValue;
		onRowHeightChange?: (value: RowHeightValue) => void;
		onRowSelect?: (rowIndex: number, checked: boolean, shiftKey: boolean) => void;
		onDataUpdate?: (params: UpdateCell | UpdateCell[]) => void;
		onRowsDelete?: (rowIndices: number[]) => void | Promise<void>;
		onColumnClick?: (columnId: string) => void;
		onCellClick?: (rowIndex: number, columnId: string, event?: MouseEvent) => void;
		onCellDoubleClick?: (rowIndex: number, columnId: string) => void;
		onCellMouseDown?: (rowIndex: number, columnId: string, event: MouseEvent) => void;
		onCellMouseEnter?: (rowIndex: number, columnId: string, event: MouseEvent) => void;
		onCellMouseUp?: () => void;
		onCellContextMenu?: (rowIndex: number, columnId: string, event: MouseEvent) => void;
		onCellEditingStart?: (rowIndex: number, columnId: string) => void;
		onCellEditingStop?: (opts?: {
			direction?: NavigationDirection;
			moveToNextRow?: boolean;
		}) => void;
		onCellsCopy?: () => void;
		onCellsCut?: () => void;
		onFilesUpload?: (params: {
			files: File[];
			rowIndex: number;
			columnId: string;
			row: TData;
		}) => Promise<FileCellData[]>;
		onFilesDelete?: (params: {
			fileIds: string[];
			rowIndex: number;
			columnId: string;
			row: TData;
		}) => void | Promise<void>;
		contextMenu?: ContextMenuState;
		onContextMenuOpenChange?: (open: boolean) => void;
		pasteDialog?: PasteDialogState;
		onPasteDialogOpenChange?: (open: boolean) => void;
		onPasteWithExpansion?: () => void;
		onPasteWithoutExpansion?: () => void;
	}
}

// ============================================
// Component Props Types
// ============================================

export interface DataGridProps<TData> {
	data: TData[];
	columns: ColumnDef<TData, unknown>[];
	readOnly?: boolean;
	height?: number;
	rowHeight?: RowHeightValue;
	autoFocus?: boolean | { rowIndex?: number; columnId?: string };
	enableColumnSelection?: boolean;
	enableSearch?: boolean;
	enablePaste?: boolean;
	overscan?: number;
	class?: string;

	// Callbacks
	onDataChange?: (data: TData[]) => void;
	onRowAdd?: (event?: MouseEvent) => Partial<CellPosition> | void | Promise<Partial<CellPosition> | void>;
	onRowsAdd?: (count: number) => void | Promise<void>;
	onRowsDelete?: (rows: TData[], rowIndices: number[]) => void | Promise<void>;
	onPaste?: (updates: UpdateCell[]) => void | Promise<void>;
	onFilesUpload?: (params: {
		files: File[];
		rowIndex: number;
		columnId: string;
		row: TData;
	}) => Promise<FileCellData[]>;
	onFilesDelete?: (params: {
		fileIds: string[];
		rowIndex: number;
		columnId: string;
		row: TData;
	}) => void | Promise<void>;

	// Snippets for customization
	header?: Snippet<[{ column: Column<TData, unknown> }]>;
	cell?: Snippet<[{ cell: Cell<TData, unknown>; row: Row<TData> }]>;
	empty?: Snippet;
	footer?: Snippet;
}

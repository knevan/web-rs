<script lang="ts" generics="TData">
    import type {CellVariantProps} from "$lib/components/data-grid/data-logic/types";
    import FlexRender from "$lib/components/ui/data-table/flex-render.svelte";
    import DataGridCellWrapper from "../parts/data-grid-cell-wrapper.svelte";

    let {
        cell,
        table,
        rowIndex,
        columnId,
        isEditing,
        isFocused,
        isSelected,
    }: CellVariantProps<TData> = $props();
</script>

<DataGridCellWrapper
        {cell}
        {table}
        {rowIndex}
        {columnId}
        {isEditing}
        {isFocused}
        {isSelected}
        class="flex size-full items-center justify-center p-0"
>
    <FlexRender
            content={cell.column.columnDef.cell}
            context={cell.getContext()}
    />
</DataGridCellWrapper>
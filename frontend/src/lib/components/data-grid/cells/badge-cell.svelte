<script lang="ts" generics="TData">
    import {cn} from "$lib/utils";
    import {getLineCount} from "../data-logic/utils";
    import {useBadgeOverflow} from "../hooks/use-badge-overflow.svelte";
    import type {CellVariantProps} from "../parts";
    import DataGridCellWrapper from "../parts/data-grid-cell-wrapper.svelte";
    import {Badge} from "$lib/components/ui/badge";

    let {
        cell,
        table,
        rowIndex,
        columnId,
        isEditing,
        isFocused,
        isSelected,
        cellValue
    }: CellVariantProps<TData> = $props();

    // Get config
    const cellOpts = $derived(cell.column.columnDef.meta?.cell);
    const options = $derived(cellOpts?.variant === 'badge' ? cellOpts.options : []);
    const variantMap = $derived(cellOpts?.variant === 'badge' ? cellOpts.variantMap : {});
    const classMap = $derived(cellOpts?.variant === 'badge' ? cellOpts.classMap : {});

    // Normalization to array
    const items = $derived.by(() => {
        if (Array.isArray(cellValue)) return cellValue;
        if (cellValue === null || cellValue === undefined || cellValue === '') return [];
        return [cellValue];
    });

    const displayItems = $derived(
        items.map((val) => {
            const strVal = String(val);
            // Label: Search in options, if not found, use the original value
            const label = options?.find((opt) => opt.value === strVal)?.label ?? strVal;
            // Variant: Search in variantMap, if not found, use default
            const badgeVariant = variantMap?.[strVal] ?? 'secondary';
            // Custom Class: Search in classMap, if not found, use empty string
            const customClass = classMap?.[strVal] ?? '';

            return {
                value: strVal,
                label,
                badgeVariant,
                customClass
            };
        })
    );

    // Overflow
    let containerRef = $state<HTMLDivElement | null>(null);
    const rowHeight = $derived(table.options.meta?.rowHeight ?? 'short');
    const lineCount = $derived(getLineCount(rowHeight));

    const badgeOverflow = useBadgeOverflow(() => ({
        items: displayItems,
        getLabel: (item) => item.label,
        containerRef: containerRef,
        lineCount: lineCount,
        cacheKeyPrefix: 'badge-cell'
    }));

    const visibleItems = $derived(badgeOverflow.value.visibleItems);
    const hiddenBadgeCount = $derived(badgeOverflow.value.hiddenCount);
</script>

<DataGridCellWrapper
        bind:wrapperRef={containerRef}
        {cell}
        {table}
        {rowIndex}
        {columnId}
        {isEditing}
        {isFocused}
        {isSelected}
>
    {#if displayItems.length > 0}
        <div class="flex flex-wrap items-center gap-1 overflow-hidden">
            {#each visibleItems as item (item.value)}
                <Badge
                        variant={item.badgeVariant}
                        class={cn(
                        "h-5 shrink-0 px-1.5 text-xs font-normal whitespace-nowrap border-transparent",
                        item.customClass
                    )}
                >
                    {item.label}
                </Badge>
            {/each}
            {#if hiddenBadgeCount > 0}
                <Badge variant="outline" class="h-5 shrink-0 px-1.5 text-muted-foreground text-xs">
                    +{hiddenBadgeCount}
                </Badge>
            {/if}
        </div>
    {/if}
</DataGridCellWrapper>
<script lang="ts">
    import {apiFetch} from "$lib/store/auth";
    import {toast} from "svelte-sonner";
    import ConfirmationAlert from "./ConfirmationAlert.svelte";
    import Badge from "./ui/badge/badge.svelte";
    import Pagination from "./Pagination.svelte";
    import {useWindowSize} from "./data-grid/hooks/use-window-size.svelte";
    import type {ColumnDef} from "@tanstack/table-core";
    import {renderSnippet} from "./data-grid/data-logic";
    import {useDataGrid} from "./data-grid/parts";
    import DataGrid from "./data-grid/parts/data-grid.svelte";
    import Button from "./ui/button/button.svelte";

    type Report = {
        id: number;
        reporter_username: string;
        reporter_id: number;
        created_at: string;
        reason: string;
        chapter_id: number | null;
        chapter_number: number | null;
        chapter_series_title: string | null;
        comment_id: number | null;
        comment_preview: string | null;
    }

    let {rowsPerPage = 25} = $props();

    let reports = $state<Report[]>([]);
    let isLoading = $state(true);
    let isResolving = $state(false);
    let resolveReportId = $state<number | null>(null);

    let currentPage = $state(1);
    let totalItems = $derived(0);
    let totalPages = $derived(Math.ceil(totalItems / rowsPerPage));

    // Layout Table
    const windowSize = useWindowSize({defaultHeight: 600});
    const gridHeight = $derived(Math.max(400, windowSize.height - 250));

    // Column Definitions
    const columns: ColumnDef<Report, unknown>[] = [
        {
            id: 'type',
            header: 'Type',
            size: 100,
            cell: ({row}) => renderSnippet(typeSnippet, {reports: row.original}),
            meta: {
                headerOptions: {
                    showDropdown: false
                },
                cell: {
                    variant: 'actions'
                }
            }
        },
        {
            id: 'target',
            header: 'Target Content',
            size: 300,
            cell: ({row}) => renderSnippet(targetSnippet, {reports: row.original}),
            meta: {
                headerOptions: {
                    showDropdown: false
                },
                cell: {
                    variant: 'actions'
                }
            }
        },
        {
            accessorKey: 'reporter_username',
            header: 'Reporter',
            meta: {
                headerOptions: {
                    showDropdown: false,
                    showIcon: false
                },
                cell: {
                    variant: 'short-text'
                }
            }
        },
        {
            accessorKey: 'created_at',
            header: 'Created Date',
            cell: ({getValue}) => formatDate(getValue() as string),
            meta: {
                headerOptions: {
                    showDropdown: false,
                },
                cell: {
                    variant: 'date'
                }
            }
        },
        {
            id: 'actions',
            header: 'Actions',
            cell: ({row}) => renderSnippet(actionsSnippet, {reports: row.original}),
            meta: {
                headerOptions: {
                    showDropdown: false
                },
                cell: {
                    variant: 'actions'
                }
            }
        }
    ]

    const {table, ...dataGridProps} = useDataGrid({
        columns,
        data: () => reports,
        onDataChange: (newData) => {
            reports = newData
        },
        getRowId: (row) => String(row.id),
        readOnly: true,
        enableColumnSelection: false,
        enableSearch: false,
    });

    // Helper function
    function formatReason(reason: string) {
        return reason.split('_').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(' ');
    }

    function formatDate(dataStr: string) {
        return new Date(dataStr).toLocaleString();
    }

    async function loadReports(page: number, limit: number) {
        isLoading = true;
        try {
            const url = new URL('/api/admin/reports/list', window.location.origin);
            url.searchParams.set('page', String(page));
            url.searchParams.set('pageSize', String(limit));

            const response = await apiFetch(url.href);
            if (!response.ok) throw new Error("Failed to fetch reports");

            const result = await response.json();

            if (result.status === 'success' && result.data) {
                reports = result.data.items;
                totalItems = result.data.totalItems;
            } else {
                reports = [];
                totalItems = 0;
            }
        } catch (error) {
            console.log(error);
            toast.error("Failed")
        } finally {
            isLoading = false;
        }
    }

    async function handleResolveConfirm() {
        if (!resolveReportId) return;
        isResolving = true;

        const resolvePromise = async () => {
            const response = await apiFetch(`/api/admin/reports/resolve/${resolveReportId}`, {
                method: "DELETE",
            });

            if (!response.ok) throw new Error("Failed to resolve");
            return "Resolved";
        }

        toast.promise(resolvePromise(), {
            loading: "Resolving report...",
            success: () => {
                loadReports(currentPage, rowsPerPage);
                return "Report resolved successfully";
            },
            error: "Failed to resolve report",
            finally: () => {
                isResolving = false;
                resolveReportId = null;
            }
        });
    }

    function handleResolveClose() {
        resolveReportId = null;
    }

    $effect(() => {
        loadReports(currentPage, rowsPerPage);
    })
</script>

{#if resolveReportId}
    <ConfirmationAlert open={!!resolveReportId}
                       title="Resolve report?"
                       message="Are you sure you want to resolve this report?"
                       onConfirm={handleResolveConfirm}
                       onCancel={handleResolveClose}
    />
{/if}

{#snippet typeSnippet({reports}: { reports: Report})}
    <div class="flex justify-center">
        <Badge variant={reports.chapter_id ? "default" : "secondary"}>
            {reports.chapter_id ? "Chapter" : "Comment"}
        </Badge>
    </div>
{/snippet}

{#snippet targetSnippet({reports}: { reports: Report})}
    <span class="line-clamp-2">
        {#if reports.chapter_id}
            {reports.chapter_number} {reports.chapter_series_title}
        {:else}
            {reports.comment_preview}
        {/if}
    </span>
{/snippet}

{#snippet actionsSnippet({reports}: { reports: Report})}
    <div class="flex justify-center">
        <Button
                variant="destructive"
                size="sm"
                onclick={() => resolveReportId = reports.id}
                disabled={isResolving}
        >
            Resolve
        </Button>
    </div>
{/snippet}

<div class="space-y-2 flex flex-col w-full">
    <DataGrid
            {...dataGridProps}
            {table}
            height={gridHeight}
            class="w-full"
            isLoading={isLoading}
            loadingMessage="Loading reports list..."
            emptyMessage="No reports found. Great job!"
    />

    {#if totalPages > 0}
        <div class="flex justify-center mt-4">
            <Pagination
                    bind:currentPage={currentPage}
                    totalPages={totalPages}
            />
        </div>
    {/if}
</div>


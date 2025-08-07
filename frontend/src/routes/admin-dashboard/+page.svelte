<svelte:head>
    <title>Admin Dashboard</title>
</svelte:head>

<script>
    import AddSeries from "$lib/components/AddSeries.svelte";
    import AdminSeriesTable from "$lib/components/AdminSeriesTable.svelte";
    import {Button} from "$lib/components/ui/button/index.js";
    import {FilePlus2Icon} from "@lucide/svelte";
    import SeriesCategoryTag from "$lib/components/SeriesCategoryTag.svelte";

    // Example data that would typically come from an API
    let activeTab = $state('series');
    let rowsPerPage = $state(20);
</script>

<div class="min-w-full p-1 md:p-0">
    <h1 class="text-2xl md:text-3xl font-bold mb-6 text-center text-foreground p-1">
        Admin Dashboard
    </h1>

    <div class="border-border px-0">
        <nav class="flex -mb-px space-x-1" aria-label="Tabs">
            <button
                    onclick={() => activeTab = 'series'}
                    class={[
                        'py-3 px-4 font-medium text-sm transition-colors cursor-pointer',
                        activeTab === 'series'
                        ? 'bg-card border-t border-x border-border rounded-t-lg text-primary'
                        : 'border-transparent text-muted-foreground'
                    ]}
            >
                List Manga
            </button>
            <button
                    onclick={() => activeTab = 'users'}
                    class={[
                        'py-3 px-4 font-medium text-sm transition-colors cursor-pointer',
                        activeTab === 'users'
                        ? 'bg-card border-t border-x border-border rounded-t-lg text-primary'
                        : 'border-transparent text-muted-foreground'
                    ]}
            >
                User
            </button>
        </nav>
    </div>

    <div class="bg-card border-x border-b border-t border-border rounded-b-lg rounded-tr-lg p-1 md:p-0.5">
        {#if activeTab === 'series'}
            <div class="space-y-4">
                <div class="flex justify-between items-center">
                    <h2 class="text-xl font-semibold text-foreground p-2">Series List</h2>
                    <div class="flex items-center space-x-2">
                        <SeriesCategoryTag/>
                        <AddSeries>
                            <Button class="cursor-pointer">
                                <FilePlus2Icon class="mr-2 h-4 w-4"/>
                                Add Series
                            </Button>
                        </AddSeries>
                    </div>
                </div>

                <div class="flex justify-end -mb-3">
                    <div class="flex items-center space-x-1">
                        <select id="rows-per-page"
                                bind:value={rowsPerPage}
                                class="h-10 w-[65px] rounded-md border border-border bg-background px-3 py-2 text-sm cursor-pointer
                                text-foreground ring-offset-background focus:outline-none focus:ring-1 focus:ring-ring focus:ring-offset-1"
                        >
                            <option value={20}>20</option>
                            <option value={50}>50</option>
                            <option value={75}>75</option>
                        </select>
                    </div>
                </div>

                <AdminSeriesTable {rowsPerPage}/>
            </div>
        {:else if activeTab === 'users'}
            <div class="space-y-4">
                <div class="flex justify-between items-center">
                    <h2 class="text-xl font-semibold text-foreground">User List</h2>
                    <Button>Add User</Button>
                </div>
                <div class="border rounded-lg p-12 text-center bg-background">
                    <p class="text-muted-foreground">Tabel untuk manajemen user akan
                        ditampilkan di sini.</p>
                </div>
            </div>
        {/if}
    </div>
</div>
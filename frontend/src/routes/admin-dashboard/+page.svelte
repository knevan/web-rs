<svelte:head>
    <title>Admin Dashboard</title>
</svelte:head>

<script lang="ts">
    import AddSeries from "$lib/components/AddSeries.svelte";
    import AdminSeriesTable from "$lib/components/AdminSeriesTable.svelte";
    import AdminUserTable from "$lib/components/AdminUserTable.svelte";
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Select from "$lib/components/ui/select/index.js";
    import {FilePlus2Icon} from "@lucide/svelte";
    import {Search} from "@lucide/svelte";
    import SeriesCategoryTag from "$lib/components/SeriesCategoryTag.svelte";
    import {Input} from "$lib/components/ui/input/index.js";

    // Example data that would typically come from an API
    let activeTab = $state('series');
    let rowsPerPage = $state(20);
    // The custom Select Layout works with string values.
    // We create a string version of rowsPerPage for the Layout.
    let rowsPerPageString = $derived(rowsPerPage.toString());
    let searchQuery = $state('');
    let debounceQuery = $state('');
    let debounceTimer: number;

    // This effect syncs the string value from the Select Layout
    // back to the original numeric rowsPerPage state.
    $effect(() => {
        const parsedValue = parseInt(rowsPerPageString, 10);
        if (!isNaN(parsedValue)) {
            rowsPerPage = parsedValue;
        }
    });

    $effect(() => {
        const currentQuery = searchQuery;
        clearTimeout(debounceTimer);

        debounceTimer = setTimeout(() => {
            debounceQuery = currentQuery;
        }, 500);

        return () => {
            clearTimeout(debounceTimer);
        };
    });
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

                <div class="flex justify-between -mb-3">
                    <div class="relative text-gray-800 dark:text-gray-200">
                        <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"/>
                        <Input
                                type="search"
                                bind:value={searchQuery}
                                class="h-10 w-full max-w-xs pl-8"
                        />
                    </div>
                    <div class="flex items-center space-x-1">
                        <Select.Root type="single" bind:value={rowsPerPageString}>
                            <Select.Trigger
                                    class="h-10 w-[60px] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus:outline-none focus:ring-1 focus:ring-ring focus:ring-offset-1">
                                {rowsPerPage}
                            </Select.Trigger>
                            <Select.Content class="w-[60px -mt-1">
                                <Select.Item value="20">20</Select.Item>
                                <Select.Item value="50">50</Select.Item>
                                <Select.Item value="75">75</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>

                <AdminSeriesTable {rowsPerPage} searchQuery={debounceQuery}/>
            </div>
        {:else if activeTab === 'users'}
            <div class="space-y-4">
                <div class="flex justify-between items-center">
                    <h2 class="text-xl font-semibold text-foreground p-2">User List</h2>
                </div>
                <div class="flex justify-between mb-1">
                    <div class="relative text-gray-800 dark:text-gray-200">
                        <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"/>
                        <Input
                                type="search"
                                bind:value={searchQuery}
                                class="h-10 w-full max-w-xs pl-8"
                        />
                    </div>
                    <div class="flex items-center space-x-1">
                        <Select.Root type="single" bind:value={rowsPerPageString}>
                            <Select.Trigger
                                    class="h-10 w-[60px] rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus:outline-none focus:ring-1 focus:ring-ring focus:ring-offset-1">
                                {rowsPerPage}
                            </Select.Trigger>
                            <Select.Content class="w-[60px -mt-1">
                                <Select.Item value="20">20</Select.Item>
                                <Select.Item value="50">50</Select.Item>
                                <Select.Item value="75">75</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>
                <AdminUserTable {rowsPerPage} searchQuery={debounceQuery}/>
            </div>
        {/if}
    </div>
</div>
<script lang="ts">
    import ModalDialog from "$lib/components/ModalDialog.svelte";
    import {Label} from "$lib/components/ui/label";
    import {Input} from "$lib/components/ui/input/index.js";
    import {toast} from "svelte-sonner";

    // seriesId to target the correct series and onclose to handle closing the modal.
    let {seriesId, onclose}: { seriesId: number, onclose: () => void } = $props();

    let formData = $state({
        chapterNumber: '',
        newChapterUrl: '',
        newChapterTitle: '',
    });

    // State managing submission
    let isSubmitting = $state(false);
    let open = $state(true);

    // call onclose handler
    $effect(() => {
        if (!open) {
            onclose();
        }
    });

    async function handleSubmit() {
        if (isSubmitting) return;
        isSubmitting = true;

        const repairRequest = async () => {
            // Validation
            const chapterNumberFloat = parseFloat(formData.chapterNumber);
            if (isNaN(chapterNumberFloat)) {
                throw new Error('Chapter number must be a valid number');
            }
            const payload = {
                chapter_number: chapterNumberFloat,
                new_chapter_url: formData.newChapterUrl,
                new_chapter_title: formData.newChapterTitle || null,
            };
            // Send request to repair chapter endpoint
            const response = await fetch(`/api/admin/repair/chapter/${seriesId}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });
            const result = await response.json();

            if (!response.ok) {
                throw new Error(result.message || 'Failed to schedule repair chapter');
            }
            return result.message;
        };

        toast.promise(repairRequest(), {
            position: "top-center",
            richColors: true,
            duration: 3000,
            loading: `Submitting repair chapter...`,
            success: (message) => {
                setTimeout(() => {
                    open = false;
                }, 1500);
                return message;
            },
            error: (err) => {
                const message = err instanceof Error ? err.message : "Unknown error";
                return `Submissing failed: ${message}`;
            },
            finally: () => {
                isSubmitting = false;
            }
        });
    }
</script>

<ModalDialog
        bind:open={open}
        title="Repair Chapter for Series ID: {seriesId}"
        onConfirm={handleSubmit}
        confirmText={isSubmitting ? "Submitting..." : "Submit Repair Chapter"}
        disableConfirm={isSubmitting || !formData.chapterNumber || !formData.newChapterUrl}
>
    {#snippet children()}
        <form class="space-y-3 pt-1 text-gray-800 dark:text-gray-100 max-h-[75vh] overflow-y-auto pr-3">
            <div>
                <Label for="chapterNumber">
                    Chapter Number <span class="text-red-500">*</span>
                </Label>
                <Input id="chapterNumber" type="number" step="0.1"
                       bind:value={formData.chapterNumber} required/>
            </div>
            <div>
                <Label for="newChapterUrl">
                    New Chapter URL <span class="text-red-500">*</span>
                </Label>
                <Input id="newChapterUrl" type="url" bind:value={formData.newChapterUrl}
                       required/>
            </div>
            <div>
                <Label for="newChapterTitle">
                    New Chapter Title (optional)
                </Label>
                <Input id="newChapterTitle" type="text"
                       bind:value={formData.newChapterTitle}/>
            </div>
        </form>
    {/snippet}
</ModalDialog>
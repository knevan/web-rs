<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import type {Snippet} from "svelte";
    import {Button} from "$lib/components/ui/button";

    interface Props {
        open?: boolean;
        title?: string;
        description?: string;
        confirmText?: string;
        cancelText?: string;
        showCancelButton?: boolean;
        showConfirmButton?: boolean;
        dialogClass?: string;
        disableConfirm?: boolean;
        trigger?: Snippet;
        children?: Snippet;
        footer?: Snippet;
        onConfirm?: () => void;
        onCancel?: () => void;
    }

    let {
        open = $bindable(),
        title = "Dialog Title",
        description = "",
        confirmText = "Confirm",
        cancelText = "Cancel",
        showCancelButton = true,
        showConfirmButton = true,
        dialogClass = "",
        disableConfirm = false,
        trigger,
        children,
        footer,
        onConfirm = () => {
        },
        onCancel = () => {
        },
    }: Props = $props();

    // let open = $state(false);

    const hasCustomFooter = $derived(!!footer);

    if (open === undefined) {
        open = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Trigger>
        {#if trigger}
            {@render trigger()}
        {/if}
    </Dialog.Trigger>
    <Dialog.Content class={dialogClass}>
        <Dialog.Header>
            <Dialog.Title class="text-left">{title}</Dialog.Title>
            <Dialog.Description class="text-left">{description}</Dialog.Description>
        </Dialog.Header>

        {#if children}
            {@render children()}
        {/if}

        <Dialog.Footer>
            {#if hasCustomFooter}
                {#if footer}
                    {@render footer()}
                {/if}
            {:else}
                <div class="flex w-full justify-end gap-2">
                    {#if showCancelButton}
                        <Dialog.Close>
                            <Button variant="outline" onclick={onCancel}>
                                {cancelText}
                            </Button>
                        </Dialog.Close>
                    {/if}
                    {#if showConfirmButton}
                        <Button onclick={onConfirm} disabled={disableConfirm}>
                            {confirmText}
                        </Button>
                    {/if}
                </div>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
<script lang="ts">
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js"

    type Props = {
        open: boolean;
        title: string;
        message: string;
        onConfirm: () => void;
        onCancel: () => void;
    };

    let {open, title, message, onConfirm, onCancel}: Props = $props();

    function handleOpenChange(isOpen: boolean) {
        if (!isOpen) {
            onCancel();
        }
    }
</script>

<AlertDialog.Root bind:open onOpenChange={handleOpenChange}>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>{title}</AlertDialog.Title>
            <AlertDialog.Description>
                {message}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel onclick={onCancel}>
                Cancel
            </AlertDialog.Cancel>
            <AlertDialog.Action onclick={onConfirm}>
                Confirm
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
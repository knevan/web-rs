<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";

    const id = $props.id();
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-3xl">Forgot Your Password?</Card.Title>
        <Card.Description class="text-left">Please enter your email address. You will receive a link via email to create
            new password.
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <div class="grid gap-4">
            <div class="grid gap-2">
                <Label for="email-{id}">Email</Label>
                <Input id="email-{id}" type="email" placeholder="your@email.com" required/>
            </div>
            <Button type="submit" class="w-full">Reset Password</Button>
        </div>
        <div class="mt-4 text-center text-sm">
            Back to Login
            <a href="/login" class="underline"> Login </a>
        </div>
    </Card.Content>
</Card.Root>
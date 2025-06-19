<script lang="ts">
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import {Label} from "$lib/components/ui/label/index.js";

    const id = $props.id();
</script>

<Card.Root class="mx-auto w-full max-w-sm">
    <Card.Header class="text-center">
        <Card.Title class="text-3xl">Register</Card.Title>
        <Card.Description>Register New Account</Card.Description>
    </Card.Header>
    <Card.Content>
        <div class="grid gap-4">
            <div class="grid gap-2">
                <Label for="username-{id}">Username</Label>
                <Input id="username-{id}" type="username" placeholder="username" required/>
            </div>
            <div class="grid gap-2">
                <Label for="email-{id}">Email</Label>
                <Input id="email-{id}" type="email" placeholder="m@example.com" required/>
            </div>
            <div class="grid gap-2">
                <div class="flex items-center">
                    <Label for="password-{id}">Password</Label>
                </div>
                <Input id="password-{id}" type="password" required/>
            </div>
            <div class="grid gap-2">
                <div class="flex items-center">
                    <Label for="confirm-password-{id}">Confirm Password</Label>
                </div>
                <Input id="confirm-password-{id}" type="confirm-password" required/>
            </div>
            <Button type="submit" class="w-full">Register</Button>
        </div>
        <div class="mt-4 text-center text-sm">
            Already have an account?
            <a href="/login" class="underline"> Login </a>
        </div>
    </Card.Content>
</Card.Root>
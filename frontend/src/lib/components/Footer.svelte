<footer class="footer-container">
    <div class="container mx-auto px-6 py-4 text-center text-gray-500">
        <p>&copy; {new Date().getFullYear()} All right reserved.</p>
    </div>

    <div>
        <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a>
        <a href="https://svelte.dev">svelte.dev</a>
        <a href="https://svelte.dev">svelte.dev</a>
    </div>
</footer>

<style>
    footer {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: 12px;
    }

    footer a {
        font-weight: bold;
    }

    .footer-container {
        border-top: 3px solid var(--border, #f87501);
        background-color: var(--background, white);
        margin-top: calc(var(--spacing) * 8);
    }
</style>
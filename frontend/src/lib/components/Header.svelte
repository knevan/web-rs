<script>
    import image_image1 from '../images/image_image.webp'
    import {auth, logout} from "$lib/store/auth.js";
    import {page} from "$app/state";
    import {toggleTheme, theme} from "$lib/store/theme.js";

    // Icons for the theme toggle button
    const moonIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>`;
    const sunIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>`
</script>

<header class="header-container">
    <div class="corner">
        <a href="/">
            <img src={image_image1} alt="SvelteKit"/>
        </a>
    </div>

    <nav>
        <svg viewBox="0 0 2 3" aria-hidden="true">
            <path d="M0,0 L1,2 C1.5,3 1.5,3 2,3 L2,0 Z"/>
        </svg>
        <ul>
            <li aria-current={page.url.pathname === '/' ? 'page' : undefined}>
                <a href="/" class="Home">Home</a>
            </li>
            <li>
                <a href="/manga-updates" class="manga-updates">Manga Updates</a>
            </li>

            <!-- Conditionally render Admin Dashboard link -->
            {#if $auth.isAuthenticated && $auth.user?.role === 'admin-dashboard'}
                <li>
                    <a href="/admin-dashboard" class="admin-dashboard">Admin Dashboard</a>
                </li>
            {/if}
            <!-- Conditionally render Sign In or Sign Out -->
            {#if $auth.isAuthenticated}
                <li>
                    <button onclick={() => logout()} class="logout-button">Sign Out</button>
                </li>
            {:else}
                <li>
                    <a href="/login" class="login">Sign In</a>
                </li>
            {/if}
        </ul>
        <svg viewBox="0 0 2 3" aria-hidden="true">
            <path d="M0,0 L0,3 C0.5,3 0.5,3 1,2 L2,0 Z"/>
        </svg>
    </nav>

    <div class="theme-toggle">
        <button onclick={toggleTheme} aria-label="Toggle Theme">
            {#if $theme === 'light'}
                {@html moonIcon}
            {:else}
                {@html sunIcon}
            {/if}
        </button>
    </div>
</header>

<style>
    .logout-button {
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-text);
        text-decoration: none;
        font-size: inherit;
        font-family: inherit;
        padding: 0;
        margin: 0;
        display: block;
        line-height: 1;
        height: 100%;
        width: 100%;
        text-align: center;
    }

    .logout-button:hover {
        text-decoration: underline;
    }

    header {
        display: flex;
        justify-content: space-between; /* Ubah dari center ke space-between */
        align-items: center;
        padding: 0 2rem;
        margin: 0 auto; /* Pusatkan header */
        position: relative;
        height: 5rem;
    }

    .header-container {
        width: 100%;
        border-bottom: 3px solid var(--border, #f17106);
        background-color: var(--background, white);
    }

    .corner {
        position: relative;
        left: auto;
        top: auto;
        width: 3em;
        height: 3em;
    }

    .corner a {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 100%;
    }

    .corner img {
        width: 2em;
        height: 2em;
        object-fit: contain;
    }

    nav {
        display: flex;
        justify-content: center;
        --background: rgba(255, 255, 255, 0.7);
    }

    svg {
        width: 2em;
        height: 3em;
        display: block;
    }

    path {
        fill: var(--background);
    }

    ul {
        position: relative;
        padding: 0;
        margin: 0;
        height: 3em;
        display: flex;
        justify-content: center;
        align-items: center;
        list-style: none;
        background: var(--background);
        background-size: contain;
    }

    li {
        position: relative;
        height: 100%;
    }

    li[aria-current='page']::before {
        --size: 6px;
        content: '';
        width: 0;
        height: 0;
        position: absolute;
        top: 0;
        left: calc(50% - var(--size));
        border: var(--size) solid transparent;
        border-top: var(--size) solid var(--color-theme-1);
    }

    nav a {
        display: flex;
        height: 100%;
        align-items: center;
        padding: 0 0.5rem;
        color: var(--color-text);
        font-weight: 700;
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        text-decoration: none;
        transition: color 0.2s linear;
    }

    a:hover {
        color: var(--color-theme-1);
    }
</style>
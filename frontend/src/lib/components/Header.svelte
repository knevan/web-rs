<script>
    import image_image1 from '../images/image_image.webp'
    import {auth, logout} from "$lib/store/auth.js";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import {Sun, Moon, Menu} from "@lucide/svelte";
    import {toggleMode, mode} from "mode-watcher";
    import {UserPlus} from "@lucide/svelte";
    import {UserCog} from "@lucide/svelte";
</script>

<header class="flex h-20 w-full items-center justify-between border-b-[3px] border-[--border] bg-[--background] px-4">
    <div class="mx-auto flex w-full lg:max-w-5xl max-w-6xl items-center justify-between">
        <div class="flex items-center gap-6">
            <a href="/" aria-label="Home">
                <img src={image_image1} alt="Home" class="h-10 w-10 object-contain"/>
            </a>
            <nav class="hidden md:flex">
                <ul class="flex items-center gap-6 list-none m-0 p-0">
                    <li>
                        <a
                                href="/manga-updates"
                                class="font-semibold text-[--color-text] no-underline transition-colors hover:text-[--color-theme-1]"
                        >Manga Updates</a
                        >
                    </li>
                    <li>
                        <a
                                href="/browse"
                                class="font-semibold text-[--color-text] no-underline transition-colors hover:text-[--color-theme-1]"
                        >
                            Browse
                        </a>
                    </li>
                    {#if $auth.isAuthenticated && $auth.user?.role === 'admin'}
                        <li>
                            <a
                                    href="/admin-dashboard"
                                    class="font-semibold text-[--color-text] no-underline transition-colors hover:text-[--color-theme-1]"
                            >Admin Dashboard</a
                            >
                        </li>
                    {/if}
                </ul>
            </nav>
        </div>

        <div class="flex items-center gap-4">
            <button onclick={toggleMode}
                    aria-label="Toggle Theme"
                    class="transition-colors hover:text-[--color-theme-1] cursor-pointer"
            >
                {#if mode.current === 'dark'}
                    <Sun/>
                {:else}
                    <Moon/>
                {/if}
            </button>

            <div class="hidden md:flex items-center gap-6">
                <!-- Conditionally render Sign In or Sign Out -->
                {#if $auth.isAuthenticated}
                    <DropdownMenu.Root>
                        <DropdownMenu.Trigger class="cursor-pointer transition-colors hover:text-[--color-theme-1]">
                            <UserCog/>
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content>
                            <DropdownMenu.Label>My Account</DropdownMenu.Label>
                            <DropdownMenu.Separator/>
                            <DropdownMenu.Item>
                                <a href="/user/profiles" class="">User Setting</a>
                            </DropdownMenu.Item>
                            <DropdownMenu.Item>
                                <a href="/user/bookmark">Bookmark Library</a>
                            </DropdownMenu.Item>
                            <DropdownMenu.Separator/>
                            <DropdownMenu.Item class="cursor-pointer" onclick={() => logout()}>Logout
                            </DropdownMenu.Item>
                        </DropdownMenu.Content>
                    </DropdownMenu.Root>
                {:else}
                    <a
                            href="/login"
                            class="flex items-center gap-2 font-semibold text-[--color-text] no-underline transition-colors hover:text-[--color-theme-1] hover:underline"
                    >
                        <UserPlus size={20}/>
                        <span>SIGN IN</span>
                    </a>
                {/if}
            </div>

            <div class="flex md:hidden">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger class="cursor-pointer transition-colors hover:text-[--color-theme-1]">
                        <Menu/>
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Content>
                        <DropdownMenu.Item>
                            <a href="/manga-updates" class="flex items-center w-full">
                                <span>Manga Updates</span>
                            </a>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item>
                            <a href="/browse" class="flex items-center w-full">
                                <span>Browse</span>
                            </a>
                        </DropdownMenu.Item>
                        {#if $auth.isAuthenticated && $auth.user?.role === 'admin'}
                            <DropdownMenu.Item>
                                <a href="/admin-dashboard" class="flex items-center w-full">
                                    <UserCog class="mr-2 h-4 w-4"/>
                                    <span>Admin Dashboard</span>
                                </a>
                            </DropdownMenu.Item>
                        {/if}
                        <DropdownMenu.Separator/>
                        {#if $auth.isAuthenticated}
                            <DropdownMenu.Label>My Account</DropdownMenu.Label>
                            <DropdownMenu.Item>
                                <a href="/user/profiles" class="flex items-center w-full">
                                    <span>User Setting</span>
                                </a>
                            </DropdownMenu.Item>
                            <DropdownMenu.Item>
                                <a href="/user/bookmark" class="flex items-center w-full">
                                    <span>Bookmark Library</span>
                                </a>
                            </DropdownMenu.Item>
                            <DropdownMenu.Separator/>
                            <DropdownMenu.Item onclick={() => logout()} class="cursor-pointer flex items-center w-full">
                                <span>Logout</span>
                            </DropdownMenu.Item>
                        {:else}
                            <DropdownMenu.Item>
                                <a href="/login" class="flex items-center w-full">
                                    <UserPlus class="mr-2 h-4 w-4"/>
                                    <span>Sign In</span>
                                </a>
                            </DropdownMenu.Item>
                        {/if}
                    </DropdownMenu.Content>
                </DropdownMenu.Root>
            </div>
        </div>
    </div>
</header>

<style>
    header {
        display: flex;
        justify-content: space-between; /* Ubah dari center ke space-between */
        align-items: center;
        padding: 0 2rem;
        margin: 0 auto; /* Pusatkan header */
        position: relative;
        height: 5rem;
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
        background-size: contain;
    }

    li {
        position: relative;
        height: 100%;
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
<script>
    import Header from '$lib/components/Header.svelte';
    import Footer from '$lib/components/Footer.svelte';
    import '../app.css'
    /*import '.style.css';*/
    import '../app.css'
    /*import '.style.css';*/
    import {onMount} from "svelte";
    import {verifyAuth} from "$lib/store/auth.js";

    // When the app loads, check if the user has a valid session cookie
    onMount(() => {
        verifyAuth();
    });

    let {children} = $props();
</script>

<div class="app">
    <Header/>

    <main>
        {@render children()}
    </main>

    <Footer/>

</div>

<style>
    .app {
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        margin: auto;
    }

    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        padding: 1rem;
        width: 100%;
        max-width: 64rem;
        margin: 0 auto;
        box-sizing: border-box;
    }
</style>

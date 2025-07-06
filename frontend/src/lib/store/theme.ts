import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// Define the possible the values
type theme = 'light' | 'dark';

const THEME_KEY = 'theme';

// Get the initial theme from local storage or system preference
function getInitialTheme(): Theme {
	if (!browser) {
		return 'light'; // Default to light theme on server
	}

	const savedTheme = localStorage.getItem(THEME_KEY);
	if (savedTheme && (savedTheme === 'light' || savedTheme === 'dark')) {
		return savedTheme as theme;
	}

	// Check for user OS preference
	if (window.matchMedia?.('(prefers-color-scheme: dark)').matches) {
		return 'dark';
	}

	return 'light'; // Default to light theme if none found
}

const initialTheme = getInitialTheme();

// Create a writable store with specific type
export const theme = writable<Theme>(initialTheme);

// Update localStorage and <html> element's data-theme attribute when theme changes
if (browser) {
	theme.subscribe((value) => {
		localStorage.setItem(THEME_KEY, value);

		if (value === 'dark') {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});
}

// Function to toggle the theme between 'light' and 'dark'
export function toggleTheme() {
	theme.update((current) => (current === 'light' ? 'dark' : 'light'));
}

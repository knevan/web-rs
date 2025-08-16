import { writable } from 'svelte/store';
import { goto } from '$app/navigation';

// Define the shape of the user object
interface User {
	identifier: string;
	role: string;
}

// Define the shape of the auth store
interface AuthStore {
	isAuthenticated: boolean;
	user: User | null;
	error: string | null;
}

// Create initial state
const initialState: AuthStore = {
	isAuthenticated: false,
	user: null,
	error: null
};

// Create the writeable store
export const auth = writable<AuthStore>(initialState);

// Store Actions

// Login function to authenticate user with credentials
export async function login(
	identifier: string,
	password: string,
	redirectTo: string | null = null
) {
	try {
		// Clear any previous errors
		auth.update((store) => ({
			...store,
			error: null
		}));

		const response = await fetch('/api/auth/login', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				identifier,
				password
			})
		});
		if (response.ok) {
			const data = await response.json();

			// Update store with successful login
			auth.update((store) => ({
				...store,
				isAuthenticated: true,
				user: data.user,
				error: null
			}));

			// Redirect to dashboard or home page after successful
			await goto(redirectTo || '/');

			return { succeess: true };
		} else {
			// Handle login failure
			const errorData = await response.json().catch(() => ({
				message: 'Login Failed'
			}));

			auth.update((store) => ({
				...store,
				isAuthenticated: false,
				user: null,
				error: errorData.message || 'Invalid Credentials'
			}));

			return {
				success: false,
				error: errorData.message || 'Invalid Credentials'
			};
		}
	} catch (e) {
		const errorMessage = 'Failed to connect to server';

		auth.update((store) => ({
			...store,
			isAuthenticated: false,
			user: null,
			error: errorMessage
		}));

		return { success: false, error: errorMessage };
	}
}

async function refreshToken(): Promise<boolean> {
	try {
		const response = await fetch('/api/auth/refresh', {
			method: 'POST'
		});

		if (response.ok) {
			return true;
		}

		await logout();
		return false;
	} catch (error) {
		await logout();
		return false;
	}
}

export async function apiFetch(
	url: string,
	options: RequestInit = {}
): Promise<Response> {
	let response = await fetch(url, options);

	if (response.status === 401) {
		console.log('Token expired. Attempting to refresh...');

		const refreshed = await refreshToken();

		if (refreshed) {
			console.log('Token refreshed successfully. Retrying original request...');
			response = await fetch(url, options);
		} else {
			// If refresh failed, the user will be logged out by the refreshToken function.
			throw new Error('Session expired. Please log in again.');
		}
	}

	return response;
}

// Register a new user by calling the backend endpoint.
export async function register(username: string, email: string, password: string) {
	try {
		const response = await fetch('api/auth/register', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ username, email, password })
		});

		if (response.ok) {
			// Registration successful
			return { success: true };
		} else {
			// Handle registration failure
			const errorData = await response.json().catch(() => ({
				message: 'Registration failed due to server error'
			}));
			// Backend provides specific error message
			return { success: false, error: errorData.message || 'Registration failed' };
		}
	} catch (e) {
		// Handle network or server errors
		const errorMessage = 'Failed to connect to server. Please try again later.';
		return { success: false, error: errorMessage };
	}
}

// Check if a username is available by calling the backend endpoint.
// @param {string} username The username to check.
// @returns {Promise<{available: boolean, message: string}>} An object indicating availability and a message.
export async function checkUsernameAvailability(username: string) {
	try {
		const response = await fetch('/api/auth/check-username', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({ username })
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({}));
			return {
				available: false,
				message: errorData.message || 'Error checking username availability.'
			};
		}

		const data = await response.json();
		// Backend should return a Json object ( available: boolean, message: string)
		return { available: data.available, message: data.message };
	} catch (e) {
		return {
			available: false,
			message: 'Failed to connect to server. Please try again later.'
		};
	}
}

// Checks if the user is authenticated by verifying the cookie with the backend.
// This should be called in the root layout to maintain session on page loads.
export async function verifyAuth() {
	try {
		// The 'protected_handler' on the backend verifies the token cookie
		// and returns user data if valid.
		const response = await apiFetch('/api/auth/user', {
			method: 'POST' // As defined in routes.rs
		});

		if (response.ok) {
			const data = await response.json();
			auth.update((store) => ({
				...store,
				isAuthenticated: true,
				user: data.user,
				error: null
			}));
		} else {
			// If the cookie is invalid or expired, and refresh fails, log the user out.
		}
	} catch (e) {
		auth.update((store) => ({
			...store,
			isAuthenticated: false,
			user: null,
			error: 'Failed to verify authentication to server'
		}));
	}
}

//Logs out the user by calling the backend endpoint and clearing the store.
export async function logout(shouldRedirect = true) {
	try {
		await fetch('/api/auth/logout', { method: 'POST' });
	} catch (e) {
		console.error('Logout API call failed, clearing state anyway.', e);
	} finally {
		auth.set(initialState);
		if (shouldRedirect) {
			await goto('/login');
		}
	}
}

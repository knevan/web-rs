import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch, cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return { user: null };
	}

	try {
		const response = await fetch('/api/auth/user', {
			method: 'POST'
		});

		// If the backend confirms the token is valid, it returns user data.
		if (response.ok) {
			const data = await response.json();
			// Pass the user data to the page.
			return { user: data.user };
		}

		return { user: null };
	} catch (error) {
		// In case of a network error or if the backend is down.
		console.error('Server-side api check error:', error);
		return { user: null };
	}
};

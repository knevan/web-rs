function createSearchStore() {
	let isOpen = $state(false);
	console.log('DEBUG: `searchStore` initialized. `isOpen` is initially:', isOpen);

	return {
		get isOpen() {
			return isOpen;
		},
		toggle: () => {
			console.log('DEBUG: `search.toggle()` called. Current `isOpen` state:', isOpen);
			isOpen = !isOpen;
			console.log('DEBUG: `isOpen` state changed to:', isOpen);
		},
		close: () => {
			console.log('DEBUG: `search.close()` called. Current `isOpen` state:', isOpen);
			isOpen = false;
			console.log('DEBUG: `isOpen` state changed to:', isOpen);
		}
	};
}

export const search = createSearchStore();
export type Toast = {
	id: number;
	kind: 'success' | 'error';
	message: string;
};

const VISIBLE_MS = 4000;

class Toasts {
	items = $state<Toast[]>([]);
	#next = 0;

	push(kind: Toast['kind'], message: string) {
		const id = this.#next++;
		this.items.push({ id, kind, message });
		setTimeout(() => this.dismiss(id), kind === 'error' ? VISIBLE_MS * 2 : VISIBLE_MS);
	}

	dismiss(id: number) {
		this.items = this.items.filter((toast) => toast.id !== id);
	}
}

export const toasts = new Toasts();

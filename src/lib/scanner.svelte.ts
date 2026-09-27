import { isTauri } from '@tauri-apps/api/core';
import {
	cancel,
	checkPermissions,
	Format,
	requestPermissions,
	scan,
} from '@tauri-apps/plugin-barcode-scanner';

/** Packaged food carries one of these; a QR code is never a product. */
const FORMATS = [Format.EAN13, Format.EAN8, Format.UPC_A, Format.UPC_E];

const DENIED = 'Scanning needs the camera. Allow it for Mealprep in the system settings.';

/**
 * The barcode scanner, which only phones have.
 *
 * The scan runs windowed: the camera preview is drawn behind the webview, which turns
 * transparent, and ScanOverlay draws the viewfinder and the way out on top. A full-screen
 * scan would cover the webview with no way to cancel it, as the plugin draws no controls.
 */
class Scanner {
	active = $state(false);
	#stop: (() => void) | null = null;

	/** The desktop build has no plugin; a barcode can be typed into search there. */
	get available(): boolean {
		return isTauri() && /android|iphone|ipad/i.test(navigator.userAgent);
	}

	/** Resolves to the scanned code, or null when the scan was cancelled. */
	async scan(): Promise<string | null> {
		let permission = await checkPermissions();
		if (permission !== 'granted') permission = await requestPermissions();
		if (permission !== 'granted') throw new Error(DENIED);

		// On Android the plugin forgets a pending scan when cancelled instead of rejecting
		// it, so cancelling settles the scan here rather than waiting for the plugin.
		const scanning = scan({ windowed: true, formats: FORMATS }).then((scanned) => scanned.content);
		const stopped = new Promise<null>((resolve) => {
			this.#stop = () => resolve(null);
		});

		this.active = true;
		document.documentElement.classList.add('scanning');
		try {
			return await Promise.race([scanning, stopped]);
		} finally {
			scanning.catch(() => {});
			this.#stop = null;
			this.active = false;
			document.documentElement.classList.remove('scanning');
		}
	}

	async cancel() {
		if (this.#stop === null) return;
		this.#stop();
		await cancel();
	}
}

export const scanner = new Scanner();

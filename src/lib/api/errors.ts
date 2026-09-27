import { error } from '@sveltejs/kit';
import type { CommandError, CommandErrorKind } from './types';

const FALLBACK = 'Something went wrong. Please try again.';

function isCommandError(value: unknown): value is CommandError {
	return (
		typeof value === 'object' &&
		value !== null &&
		'kind' in value &&
		'message' in value &&
		typeof (value as CommandError).message === 'string'
	);
}

/**
 * A command rejects with the serialised error from src-tauri/src/error.rs. Tauri itself
 * rejects with a plain string when the arguments do not deserialise, and the API module
 * with an Error outside the app, so none of the three is assumed.
 */
export function messageOf(reason: unknown): string {
	if (isCommandError(reason) && reason.message.trim() !== '') return reason.message;
	if (reason instanceof Error && reason.message.trim() !== '') return reason.message;
	if (typeof reason === 'string' && reason.trim() !== '') return reason;
	return FALLBACK;
}

export function errorKind(reason: unknown): CommandErrorKind | undefined {
	return isCommandError(reason) ? reason.kind : undefined;
}

/** Awaits a read for a load function, turning a failure into the error page. */
export async function read<T>(pending: Promise<T>): Promise<T> {
	try {
		return await pending;
	} catch (reason) {
		error(errorKind(reason) === 'not-found' ? 404 : 500, messageOf(reason));
	}
}

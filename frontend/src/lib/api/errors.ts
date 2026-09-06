import type { ErrorResponse } from './gen/types.gen';

const FALLBACK = 'Something went wrong. Please try again.';

function isErrorResponse(value: unknown): value is ErrorResponse {
	return (
		typeof value === 'object' &&
		value !== null &&
		'message' in value &&
		typeof (value as ErrorResponse).message === 'string'
	);
}

/**
 * The API answers every failure with an ErrorResponse, and the client throws that
 * body as-is. A network failure throws an Error instead, and an empty body throws
 * `{}`, so neither is assumed to carry a message.
 */
export function messageOf(error: unknown): string {
	if (isErrorResponse(error) && error.message.trim() !== '') return error.message;
	if (error instanceof Error && error.message.trim() !== '') return error.message;
	return FALLBACK;
}

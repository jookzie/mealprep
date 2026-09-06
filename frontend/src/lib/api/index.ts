// The one module the rest of the app imports the API through. Components never call
// fetch, and nothing outside this directory reaches into gen/, which is regenerated
// from api/openapi.yaml by `mise run openapi:gen`.
import './client';

export { messageOf } from './errors';
export * from './gen/sdk.gen';
export type * from './gen/types.gen';
export { type MutationOptions, runMutation } from './mutate';
export { loadTargets } from './targets';

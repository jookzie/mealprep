import { client } from './gen/client.gen';

// The generated client already carries `/v1` from the spec's servers entry, so the
// only thing configured here is failure handling: a call that fails throws instead
// of resolving to an error object. Call sites repeat `throwOnError: true` so the
// types narrow too; this makes the runtime behaviour hold even where they forget.
client.setConfig({ throwOnError: true });

export { client };

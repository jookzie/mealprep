// The app is a static SPA: no server rendering, nothing prerendered. Every route
// inherits this, so dynamic routes need no opt-out of their own.
export const ssr = false;
export const prerender = false;

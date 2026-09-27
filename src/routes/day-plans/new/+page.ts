import { loadComposition } from '../composition';
import type { PageLoad } from './$types';

export const load: PageLoad = () => loadComposition();

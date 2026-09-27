import { loadEnergyBalance, read } from '$lib/api';
import { todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => ({
	balance: await read(loadEnergyBalance(todayIso())),
});

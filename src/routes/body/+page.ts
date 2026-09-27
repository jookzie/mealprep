import {
	listMeasurements,
	loadEnergyBalance,
	loadHealthStatus,
	loadRecovery,
	loadSleepSummary,
	loadTargets,
	loadWeightSeries,
	read,
} from '$lib/api';
import { todayIso } from '$lib/domain/week';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const today = todayIso();
	const [targets, weight, measurements, health, recovery, sleep, energy] = await read(
		Promise.all([
			loadTargets(),
			loadWeightSeries(),
			listMeasurements(),
			loadHealthStatus(),
			loadRecovery(today),
			loadSleepSummary(today),
			loadEnergyBalance(today),
		]),
	);
	return { targets, weight, measurements, health, recovery, sleep, energy };
};

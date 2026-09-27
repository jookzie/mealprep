package com.mertan.mealprep.health

import android.app.Activity
import android.content.Intent
import android.net.Uri
import androidx.activity.result.ActivityResult
import androidx.health.connect.client.HealthConnectClient
import androidx.health.connect.client.HealthConnectFeatures
import androidx.health.connect.client.PermissionController
import androidx.health.connect.client.permission.HealthPermission
import androidx.health.connect.client.records.ActiveCaloriesBurnedRecord
import androidx.health.connect.client.records.ExerciseSessionRecord
import androidx.health.connect.client.records.HeartRateVariabilityRmssdRecord
import androidx.health.connect.client.records.Record
import androidx.health.connect.client.records.RespiratoryRateRecord
import androidx.health.connect.client.records.RestingHeartRateRecord
import androidx.health.connect.client.records.SleepSessionRecord
import androidx.health.connect.client.records.TotalCaloriesBurnedRecord
import androidx.health.connect.client.request.AggregateGroupByPeriodRequest
import androidx.health.connect.client.request.ReadRecordsRequest
import androidx.health.connect.client.time.TimeRangeFilter
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.time.Instant
import java.time.LocalDate
import java.time.Period
import java.time.ZoneId
import java.time.ZoneOffset
import kotlin.reflect.KClass
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@InvokeArg
class ReadArgs {
    /** The first local day to read, as YYYY-MM-DD. */
    lateinit var from: String

    /** The last local day to read, inclusive. */
    lateinit var to: String
}

/**
 * Reads what the app needs from Health Connect and reports it as plain records.
 *
 * Nothing is summarised here: the Rust rules decide which morning a night belongs to and
 * which session is the main sleep, where they are tested. The one exception is energy and
 * exercise, which Health Connect aggregates per day itself, because only it knows the
 * priority the user set between apps that record the same activity.
 */
@TauriPlugin
class HealthConnectPlugin(private val activity: Activity) : Plugin(activity) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val permissionContract = PermissionController.createRequestPermissionResultContract()

    private val readTypes: List<KClass<out Record>> = listOf(
        SleepSessionRecord::class,
        RestingHeartRateRecord::class,
        HeartRateVariabilityRmssdRecord::class,
        RespiratoryRateRecord::class,
        ExerciseSessionRecord::class,
        ActiveCaloriesBurnedRecord::class,
        TotalCaloriesBurnedRecord::class,
    )
    private val readPermissions: Set<String> =
        readTypes.map { HealthPermission.getReadPermission(it) }.toSet()

    /** The read permissions, plus history where this Health Connect version offers it. */
    private fun requestedPermissions(client: HealthConnectClient): Set<String> {
        val history = client.features.getFeatureStatus(
            HealthConnectFeatures.FEATURE_READ_HEALTH_DATA_HISTORY,
        ) == HealthConnectFeatures.FEATURE_STATUS_AVAILABLE
        return if (history) {
            readPermissions + HealthPermission.PERMISSION_READ_HEALTH_DATA_HISTORY
        } else {
            readPermissions
        }
    }

    private fun availability(): String =
        when (HealthConnectClient.getSdkStatus(activity)) {
            HealthConnectClient.SDK_AVAILABLE -> "available"
            HealthConnectClient.SDK_UNAVAILABLE_PROVIDER_UPDATE_REQUIRED -> "update-required"
            else -> "unsupported"
        }

    private fun client(): HealthConnectClient? =
        if (availability() == "available") HealthConnectClient.getOrCreate(activity) else null

    private suspend fun granted(client: HealthConnectClient): Set<String> =
        client.permissionController.getGrantedPermissions()

    /** Runs the given block off the main thread, turning any failure into a rejection. */
    private fun launch(invoke: Invoke, block: suspend () -> Unit) {
        scope.launch {
            try {
                block()
            } catch (error: Exception) {
                invoke.reject(error.message ?: error.javaClass.simpleName, error)
            }
        }
    }

    @Command
    fun status(invoke: Invoke) {
        launch(invoke) {
            val client = client()
            val connected = client != null && granted(client).any { it in readPermissions }
            invoke.resolve(
                JSObject().put("availability", availability()).put("connected", connected),
            )
        }
    }

    @Command
    fun connect(invoke: Invoke) {
        val client = client() ?: return invoke.reject("Health Connect is not available")
        launch(invoke) {
            val requested = requestedPermissions(client)
            if (granted(client).containsAll(requested)) {
                invoke.resolve(JSObject().put("connected", true))
                return@launch
            }
            val intent = permissionContract.createIntent(activity, requested)
            withContext(Dispatchers.Main) { startActivityForResult(invoke, intent, "connected") }
        }
    }

    @ActivityCallback
    fun connected(invoke: Invoke, result: ActivityResult) {
        val client = client() ?: return invoke.reject("Health Connect is not available")
        launch(invoke) {
            val connected = granted(client).any { it in readPermissions }
            invoke.resolve(JSObject().put("connected", connected))
        }
    }

    @Command
    fun install(invoke: Invoke) {
        val uri = Uri.parse(
            "market://details?id=com.google.android.apps.healthdata&url=healthconnect%3A%2F%2Fonboarding",
        )
        activity.startActivity(
            Intent(Intent.ACTION_VIEW, uri).setPackage("com.android.vending")
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        )
        invoke.resolve()
    }

    @Command
    fun read(invoke: Invoke) {
        val args = invoke.parseArgs(ReadArgs::class.java)
        val client = client() ?: return invoke.reject("Health Connect is not available")
        launch(invoke) {
            val zone = ZoneId.systemDefault()
            val first = LocalDate.parse(args.from)
            val last = LocalDate.parse(args.to)
            // A day early, so a night that began the evening before the first day is whole.
            val range = TimeRangeFilter.between(
                first.minusDays(1).atStartOfDay(zone).toInstant(),
                last.plusDays(1).atStartOfDay(zone).toInstant(),
            )
            val allowed = granted(client)
            fun allows(type: KClass<out Record>) = HealthPermission.getReadPermission(type) in allowed

            val result = JSObject()
            result.put(
                "sleep",
                if (allows(SleepSessionRecord::class)) sleep(client, range, zone) else JSArray(),
            )
            result.put(
                "restingHeartRate",
                if (allows(RestingHeartRateRecord::class)) {
                    samples(readAll(client, RestingHeartRateRecord::class, range), zone) {
                        Triple(it.time, it.zoneOffset, it.beatsPerMinute.toDouble())
                    }
                } else JSArray(),
            )
            result.put(
                "heartRateVariability",
                if (allows(HeartRateVariabilityRmssdRecord::class)) {
                    samples(readAll(client, HeartRateVariabilityRmssdRecord::class, range), zone) {
                        Triple(it.time, it.zoneOffset, it.heartRateVariabilityMillis)
                    }
                } else JSArray(),
            )
            result.put(
                "respiratoryRate",
                if (allows(RespiratoryRateRecord::class)) {
                    samples(readAll(client, RespiratoryRateRecord::class, range), zone) {
                        Triple(it.time, it.zoneOffset, it.rate)
                    }
                } else JSArray(),
            )
            result.put("activity", activity(client, first, last, ::allows))
            invoke.resolve(result)
        }
    }

    private suspend fun <T : Record> readAll(
        client: HealthConnectClient,
        type: KClass<T>,
        range: TimeRangeFilter,
    ): List<T> {
        val records = mutableListOf<T>()
        var pageToken: String? = null
        do {
            val response = client.readRecords(
                ReadRecordsRequest(type, range, pageToken = pageToken),
            )
            records += response.records
            pageToken = response.pageToken
        } while (pageToken != null)
        return records
    }

    private fun offsetSeconds(offset: ZoneOffset?, at: Instant, zone: ZoneId): Int =
        (offset ?: zone.rules.getOffset(at)).totalSeconds

    private suspend fun sleep(
        client: HealthConnectClient,
        range: TimeRangeFilter,
        zone: ZoneId,
    ): JSArray {
        val sessions = JSArray()
        for (session in readAll(client, SleepSessionRecord::class, range)) {
            val stages = JSArray()
            for (stage in session.stages) {
                stages.put(
                    JSObject()
                        .put("startMs", stage.startTime.toEpochMilli())
                        .put("endMs", stage.endTime.toEpochMilli())
                        .put("stage", stage.stage),
                )
            }
            sessions.put(
                JSObject()
                    .put("startMs", session.startTime.toEpochMilli())
                    .put("startOffset", offsetSeconds(session.startZoneOffset, session.startTime, zone))
                    .put("endMs", session.endTime.toEpochMilli())
                    .put("endOffset", offsetSeconds(session.endZoneOffset, session.endTime, zone))
                    .put("stages", stages),
            )
        }
        return sessions
    }

    private fun <T : Record> samples(
        records: List<T>,
        zone: ZoneId,
        read: (T) -> Triple<Instant, ZoneOffset?, Double>,
    ): JSArray {
        val samples = JSArray()
        for (record in records) {
            val (at, offset, value) = read(record)
            samples.put(
                JSObject()
                    .put("atMs", at.toEpochMilli())
                    .put("offset", offsetSeconds(offset, at, zone))
                    .put("value", value),
            )
        }
        return samples
    }

    private suspend fun activity(
        client: HealthConnectClient,
        first: LocalDate,
        last: LocalDate,
        allows: (KClass<out Record>) -> Boolean,
    ): JSArray {
        val active = allows(ActiveCaloriesBurnedRecord::class)
        val total = allows(TotalCaloriesBurnedRecord::class)
        val exercise = allows(ExerciseSessionRecord::class)
        val metrics = buildSet {
            if (active) add(ActiveCaloriesBurnedRecord.ACTIVE_CALORIES_TOTAL)
            if (total) add(TotalCaloriesBurnedRecord.ENERGY_TOTAL)
            if (exercise) add(ExerciseSessionRecord.EXERCISE_DURATION_TOTAL)
        }
        val days = JSArray()
        if (metrics.isEmpty()) return days

        val buckets = client.aggregateGroupByPeriod(
            AggregateGroupByPeriodRequest(
                metrics = metrics,
                timeRangeFilter = TimeRangeFilter.between(
                    first.atStartOfDay(),
                    last.plusDays(1).atStartOfDay(),
                ),
                timeRangeSlicer = Period.ofDays(1),
            ),
        )
        for (bucket in buckets) {
            val day = JSObject().put("date", bucket.startTime.toLocalDate().toString())
            if (active) {
                bucket.result[ActiveCaloriesBurnedRecord.ACTIVE_CALORIES_TOTAL]
                    ?.let { day.put("activeKcal", it.inKilocalories) }
            }
            if (total) {
                bucket.result[TotalCaloriesBurnedRecord.ENERGY_TOTAL]
                    ?.let { day.put("totalKcal", it.inKilocalories) }
            }
            if (exercise) {
                bucket.result[ExerciseSessionRecord.EXERCISE_DURATION_TOTAL]
                    ?.let { day.put("exerciseMinutes", it.toMinutes().toDouble()) }
            }
            days.put(day)
        }
        return days
    }
}

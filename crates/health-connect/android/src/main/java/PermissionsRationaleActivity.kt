package com.mertan.mealprep.health

import android.app.Activity
import android.os.Bundle
import android.widget.ScrollView
import android.widget.TextView

/**
 * What Health Connect shows when the user asks why the app wants access. It must exist for
 * the permission screen to open at all.
 */
class PermissionsRationaleActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val padding = (24 * resources.displayMetrics.density).toInt()
        val text = TextView(this).apply {
            setPadding(padding, padding, padding, padding)
            textSize = 16f
            text = RATIONALE
        }
        setContentView(ScrollView(this).apply { addView(text) })
    }

    private companion object {
        const val RATIONALE =
            "Mealprep reads sleep, resting heart rate, heart rate variability, respiratory " +
                "rate, workouts and energy burned from Health Connect.\n\n" +
                "It uses them to show your recovery against your own normal range, how " +
                "regularly you sleep, which habits go with better mornings, and how much " +
                "energy your weight trend says you burn.\n\n" +
                "The data is copied into Mealprep's database on this device only. Nothing " +
                "is sent anywhere, and Mealprep never writes to Health Connect."
    }
}

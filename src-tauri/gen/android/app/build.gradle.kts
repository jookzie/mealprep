import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

// The release keystore and its passwords stay out of the repository: they are read
// from an untracked keystore.properties next to the Gradle project. When it is
// absent the release build type is left unsigned rather than falling back to the
// debug key, which would produce an APK that looks shippable and is not.
val keystoreProperties = Properties().apply {
    val propFile = rootProject.file("keystore.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

/*
 * CI builds two release environments out of these same sources — production and
 * acceptance — and they differ only in application id, launcher label and version
 * code. All three come from the environment so that an empty environment is exactly
 * the production build this file has always described: `mise run android:build:release`
 * is unchanged, and nothing about the install on the phone moves.
 *
 * The application id is what keeps them apart. Android gives every id its own private
 * data directory, so the acceptance install cannot see production's database even by
 * accident, and the two coexist. Production's id must therefore never pick up a suffix:
 * a changed id (like a changed signing key) is a different app, and installing it means
 * uninstalling the old one, which deletes the database.
 *
 * versionCode scheme, set by the release workflow from its run number:
 *
 *     production   1_000_000 + run_number
 *     acceptance               run_number
 *
 * Run numbers only ever increase, so each environment's series is monotonic — Android
 * refuses an update whose code went backwards — and the production offset keeps a
 * production code above every acceptance code that will plausibly be built, so the two
 * series can never be confused for one another. Unset, versionCode falls back to
 * tauri.properties as before, which is what local builds use.
 */
val releaseApplicationIdSuffix: String? = System.getenv("MEALPREP_APPLICATION_ID_SUFFIX")
val releaseLabelSuffix: String? = System.getenv("MEALPREP_LABEL_SUFFIX")
val releaseVersionCode: Int? = System.getenv("MEALPREP_VERSION_CODE")?.toInt()

// Gradle cannot append to a string resource, so a suffixed label is assembled from the
// same word strings.xml holds. Without a suffix the placeholders stay the resource
// references the manifest carried before.
val appLabel = "Mealprep"

android {
    compileSdk = 36
    namespace = "com.mertan.mealprep"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        manifestPlaceholders["appLabel"] = "@string/app_name"
        manifestPlaceholders["activityLabel"] = "@string/main_activity_title"
        applicationId = "com.mertan.mealprep"
        minSdk = 24
        targetSdk = 36
        versionCode = releaseVersionCode
            ?: tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    signingConfigs {
        if (keystoreProperties.containsKey("storeFile")) {
            create("release") {
                storeFile = file(keystoreProperties.getProperty("storeFile"))
                storePassword = keystoreProperties.getProperty("storePassword")
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
            }
        }
    }
    buildTypes {
        getByName("debug") {
            applicationIdSuffix = ".debug"
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            // Absent variables leave this block exactly as it was: the production install.
            releaseApplicationIdSuffix?.let { applicationIdSuffix = it }
            releaseLabelSuffix?.let {
                manifestPlaceholders["appLabel"] = appLabel + it
                manifestPlaceholders["activityLabel"] = appLabel + it
            }
            signingConfig = signingConfigs.findByName("release")
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")
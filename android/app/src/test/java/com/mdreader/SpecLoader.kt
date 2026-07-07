package com.mdreader

import org.json.JSONObject

/**
 * Loads shared golden test cases from the JSON files under shared/specs so all
 * three platforms (Android/macOS/Linux) assert against the SAME input→output
 * contract.
 *
 * The spec files live in the monorepo at shared/specs and are wired onto the
 * JVM test classpath (see app/build.gradle.kts test resources srcDir), so they
 * load as classpath resources regardless of the test working directory.
 *
 * Each spec is an object with a "cases" array. Callers parse the fields they
 * need.
 */
object SpecLoader {

    /** Returns the raw case objects for [name] (e.g. "content_hash"). */
    fun cases(name: String): List<JSONObject> {
        val res = requireNotNull(SpecLoader::class.java.classLoader) { "no classLoader" }
            .getResourceAsStream("$name.json")
        requireNotNull(res) { "spec resource not found: $name.json (is shared/specs on the test classpath?)" }
        val text = res.bufferedReader().use { it.readText() }
        return JSONObject(text).getJSONArray("cases").let { arr ->
            (0 until arr.length()).map { arr.getJSONObject(it) }
        }
    }
}

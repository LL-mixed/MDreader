package com.mdreader.data

import android.content.Context
import android.content.SharedPreferences

/**
 * Per-content-hash dark-mode override map, persisted in SharedPreferences.
 * Mirrors macOS `ThemeStore` (`~/.mdreader/theme.json`) / Linux `theme_store`.
 * A doc lands here only when the user toggles its theme; absence means "follow
 * the global default" ([SettingsStore.themePref]).
 *
 * Note: SharedPreferences only stores a fixed set of types, so the hash→bool
 * map is flattened into one boolean key per hash (`"dark_<hash>"`).
 */
class ThemeStore(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences("mdreader_theme", Context.MODE_PRIVATE)

    /** The per-doc dark override, or null if the doc has no override. */
    fun isDarkFor(hash: String): Boolean? {
        if (!prefs.contains(key(hash))) return null
        return prefs.getBoolean(key(hash), false)
    }

    /** Sets/clears the per-doc override. Pass null to "unpin" (follow default again). */
    fun setDark(dark: Boolean?, hash: String) {
        val e = prefs.edit()
        if (dark == null) e.remove(key(hash)) else e.putBoolean(key(hash), dark)
        e.apply()
    }

    private fun key(hash: String) = "dark_$hash"
}

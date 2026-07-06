package com.mdreader.data

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/** Global default theme preference. Mirrors macOS `ThemePref` / Linux `ThemePref`. */
enum class ThemePref { SYSTEM, LIGHT, DARK }

/**
 * User-level settings, persisted in SharedPreferences. Mirrors macOS `AppSettings`
 * / Linux `settings_store`. [themePref] is exposed as Compose state so settings-UI
 * edits drive recomposition directly while also persisting.
 */
class SettingsStore(context: Context) {
    private val prefs = context.getSharedPreferences("mdreader_settings", Context.MODE_PRIVATE)

    private var _themePref by mutableStateOf(loadThemePref())
    private var _editorCommand by mutableStateOf(prefs.getString(KEY_EDITOR, "") ?: "")

    /** Default theme for docs without a per-doc override. */
    val themePref: ThemePref get() = _themePref

    /** External editor command (reserved — not wired yet). */
    val editorCommand: String get() = _editorCommand

    /** Updates the default theme preference and persists it. */
    fun updateThemePref(pref: ThemePref) {
        _themePref = pref
        prefs.edit().putString(KEY_THEME, pref.name).apply()
    }

    /** Updates the editor command and persists it. */
    fun updateEditorCommand(value: String) {
        _editorCommand = value
        prefs.edit().putString(KEY_EDITOR, value).apply()
    }

    private fun loadThemePref(): ThemePref =
        prefs.getString(KEY_THEME, null)?.let { name ->
            runCatching { ThemePref.valueOf(name) }.getOrNull()
        } ?: ThemePref.SYSTEM

    private companion object {
        const val KEY_THEME = "theme_pref"
        const val KEY_EDITOR = "editor_command"
    }
}

package com.mdreader.util

import com.mdreader.data.ThemePref

/**
 * Pure theme-resolution rule (unit-testable on the JVM without Android framework).
 * Mirrors macOS `ReaderModel.resolveDark` / Linux `util::theme::resolve_dark`:
 * a per-document override always wins; otherwise the global [ThemePref] decides,
 * where [ThemePref.SYSTEM] follows [systemDark].
 */
fun resolveDark(perDoc: Boolean?, pref: ThemePref, systemDark: Boolean): Boolean =
    perDoc ?: when (pref) {
        ThemePref.SYSTEM -> systemDark
        ThemePref.LIGHT -> false
        ThemePref.DARK -> true
    }

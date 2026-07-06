package com.mdreader.util

import com.mdreader.data.ThemePref
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Unit tests for [resolveDark]. Pure JVM logic — mirrors macOS `resolveDark` and
 * Linux `resolve_dark` test cases so all three platforms agree on the rule.
 */
class ThemeResolverTest {

    @Test fun perDocOverrideWinsOverEverything() {
        assertEquals(true, resolveDark(true, ThemePref.LIGHT, systemDark = false))
        assertEquals(false, resolveDark(false, ThemePref.DARK, systemDark = true))
    }

    @Test fun systemPrefFollowsSystemDark() {
        assertEquals(true, resolveDark(null, ThemePref.SYSTEM, systemDark = true))
        assertEquals(false, resolveDark(null, ThemePref.SYSTEM, systemDark = false))
    }

    @Test fun lightPrefAlwaysLight() {
        assertEquals(false, resolveDark(null, ThemePref.LIGHT, systemDark = true))
        assertEquals(false, resolveDark(null, ThemePref.LIGHT, systemDark = false))
    }

    @Test fun darkPrefAlwaysDark() {
        assertEquals(true, resolveDark(null, ThemePref.DARK, systemDark = true))
        assertEquals(true, resolveDark(null, ThemePref.DARK, systemDark = false))
    }

    @Test fun nullPerDocFallsThroughToPref() {
        // Same as systemPrefFollowsSystemDark but makes the null-fallthrough explicit.
        assertEquals(false, resolveDark(null, ThemePref.SYSTEM, systemDark = false))
    }

    @Test fun perDocOverrideWinsOverSystemPref() {
        assertEquals(false, resolveDark(false, ThemePref.SYSTEM, systemDark = true))
    }
}

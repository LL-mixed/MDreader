package com.mdreader.data

import android.content.Context
import android.content.SharedPreferences

/**
 * Persists the last-opened document id so the app can restore it on relaunch.
 * Mirrors macOS `SessionStore` / Linux `session_store`. Stored in a dedicated
 * SharedPreferences file so it survives process death but stays app-private.
 */
class SessionStore(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences("mdreader_session", Context.MODE_PRIVATE)

    /** The cached-doc id to reopen on next launch, or null to start at the library. */
    var lastDocId: Long?
        get() {
            val id = prefs.getLong(KEY, -1L)
            return if (id == -1L) null else id
        }
        set(value) {
            val e = prefs.edit()
            if (value == null) e.remove(KEY) else e.putLong(KEY, value)
            e.apply()
        }

    private companion object {
        const val KEY = "last_doc_id"
    }
}

package com.mdreader

import android.app.Application
import com.mdreader.data.DocRepository
import com.mdreader.data.SessionStore
import com.mdreader.data.SettingsStore
import com.mdreader.data.ThemeStore
import com.mdreader.data.db.AppDatabase

/** Process-wide singletons: the Room database, repository, and persistence stores. */
class App : Application() {
    val database by lazy { AppDatabase.get(this) }
    val repository by lazy { DocRepository(database.cachedDocDao(), this) }
    val sessionStore by lazy { SessionStore(this) }
    val themeStore by lazy { ThemeStore(this) }
    val settingsStore by lazy { SettingsStore(this) }
}

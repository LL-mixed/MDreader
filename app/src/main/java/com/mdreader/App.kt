package com.mdreader

import android.app.Application
import com.mdreader.data.DocRepository
import com.mdreader.data.db.AppDatabase

/** Process-wide singletons: the Room database and the document repository. */
class App : Application() {
    val database by lazy { AppDatabase.get(this) }
    val repository by lazy { DocRepository(database.cachedDocDao(), this) }
}

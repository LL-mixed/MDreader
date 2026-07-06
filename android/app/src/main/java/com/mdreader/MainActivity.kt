package com.mdreader

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.lifecycleScope
import com.mdreader.data.LoadedDocument
import com.mdreader.data.MarkdownSources
import com.mdreader.ui.LibraryScreen
import com.mdreader.ui.MDreaderTheme
import com.mdreader.ui.ReaderFromCache
import com.mdreader.ui.ReaderScreen
import com.mdreader.ui.SettingsScreen
import kotlinx.coroutines.launch

private enum class Screen { Library, Reader, Settings }

class MainActivity : ComponentActivity() {

    private val app by lazy { application as App }
    private val repository by lazy { app.repository }
    private val sessionStore by lazy { app.sessionStore }
    private val themeStore by lazy { app.themeStore }
    private val settingsStore by lazy { app.settingsStore }

    private var screen by mutableStateOf(Screen.Library)
    private var readerDocId by mutableStateOf<Long?>(null)
    private var readerDirect by mutableStateOf<LoadedDocument?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        if (!handleIntent(intent)) {
            restoreSession()
        }
        setContent {
            MDreaderTheme {
                BackHandler(enabled = screen != Screen.Library) { navigateToLibrary() }
                when (screen) {
                    Screen.Library -> LibraryScreen(
                        repository = repository,
                        onOpen = { id -> navigateToReader(id) },
                        onOpenSample = { openSample() },
                    )
                    Screen.Reader -> {
                        val direct = readerDirect
                        if (direct != null) {
                            ReaderScreen(
                                title = direct.title,
                                markdown = direct.markdown,
                                themeStore = themeStore,
                                settingsStore = settingsStore,
                                onOpenSettings = { screen = Screen.Settings },
                                onBack = ::navigateToLibrary,
                            )
                        } else {
                            readerDocId?.let { id ->
                                ReaderFromCache(
                                    docId = id,
                                    repository = repository,
                                    themeStore = themeStore,
                                    settingsStore = settingsStore,
                                    onOpenSettings = { screen = Screen.Settings },
                                    onBack = ::navigateToLibrary,
                                )
                            }
                        }
                    }
                    Screen.Settings -> SettingsScreen(
                        settingsStore = settingsStore,
                        onBack = { screen = Screen.Library },
                    )
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    /** @return true if an incoming VIEW/SEND intent was consumed (a doc is now open). */
    private fun handleIntent(intent: Intent?): Boolean {
        if (intent == null) return false
        when (intent.action) {
            Intent.ACTION_VIEW -> {
                val uri = intent.data
                if (uri != null) { openFromUri(uri); return true }
            }
            Intent.ACTION_SEND -> {
                // Shared file (EXTRA_STREAM) — treat like a VIEW on that uri.
                val stream = intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
                if (stream != null) { openFromUri(stream); return true }
                // Shared plain text (EXTRA_TEXT) — open it directly.
                val text = intent.getStringExtra(Intent.EXTRA_TEXT)
                if (!text.isNullOrBlank()) {
                    openFromText(text, getString(R.string.app_name))
                    return true
                }
            }
        }
        return false
    }

    /** Reopens the last-read cached doc on a normal launch (no explicit uri). */
    private fun restoreSession() {
        val id = sessionStore.lastDocId ?: return
        lifecycleScope.launch {
            // Only restore if the cached row still exists; a stale id is cleared.
            if (repository.loadContent(id) != null) {
                readerDocId = id
                readerDirect = null
                screen = Screen.Reader
            } else {
                sessionStore.lastDocId = null
            }
        }
    }

    private fun openFromUri(uri: Uri) {
        // Best-effort persistable permission so refreshFromSource can re-read the
        // source later (works for ACTION_OPEN_DOCUMENT uris; ACTION_VIEW-forwarded
        // uris don't carry a persistable grant and this throws — we ignore that).
        runCatching {
            contentResolver.takePersistableUriPermission(
                uri, Intent.FLAG_GRANT_READ_URI_PERMISSION
            )
        }
        val title = MarkdownSources.displayName(this, uri).ifBlank { getString(R.string.app_name) }
        val body = MarkdownSources.readText(this, uri)
        readerDirect = LoadedDocument(
            title = title,
            markdown = body ?: "# 无法打开\n\n读取该 Markdown 文件失败。",
        )
        readerDocId = null
        screen = Screen.Reader
        if (body != null) {
            lifecycleScope.launch {
                val cached = repository.cache(title, body, uri.toString())
                sessionStore.lastDocId = cached.id
            }
        }
    }

    private fun openSample() {
        val markdown = assets.open("sample.md").bufferedReader().use { it.readText() }
        openFromText(markdown, getString(R.string.app_name))
    }

    /** Opens shared plain text (e.g. ACTION_SEND EXTRA_TEXT) directly, no source file. */
    private fun openFromText(markdown: String, title: String) {
        readerDirect = LoadedDocument(title, markdown)
        readerDocId = null
        screen = Screen.Reader
        // No sourceUri to cache against, so skip persistence for text shares.
    }

    private fun navigateToReader(id: Long) {
        readerDocId = id
        readerDirect = null
        screen = Screen.Reader
        sessionStore.lastDocId = id
    }

    private fun navigateToLibrary() {
        screen = Screen.Library
        readerDirect = null
        readerDocId = null
    }
}

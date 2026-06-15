package com.mdreader

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
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
import kotlinx.coroutines.launch

private enum class Screen { Library, Reader }

class MainActivity : ComponentActivity() {

    private val repository by lazy { (application as App).repository }

    private var screen by mutableStateOf(Screen.Library)
    private var readerDocId by mutableStateOf<Long?>(null)
    private var readerDirect by mutableStateOf<LoadedDocument?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        handleIntent(intent)
        setContent {
            MDreaderTheme {
                BackHandler(enabled = screen == Screen.Reader) { navigateToLibrary() }
                when (screen) {
                    Screen.Library -> LibraryScreen(
                        repository = repository,
                        onOpen = { id ->
                            readerDocId = id
                            readerDirect = null
                            screen = Screen.Reader
                        },
                        onOpenSample = { openSample() },
                    )
                    Screen.Reader -> {
                        val direct = readerDirect
                        if (direct != null) {
                            ReaderScreen(direct.title, direct.markdown, onBack = ::navigateToLibrary)
                        } else {
                            readerDocId?.let { id ->
                                ReaderFromCache(id, repository, onBack = ::navigateToLibrary)
                            }
                        }
                    }
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        val uri = intent?.data
        if (uri != null && intent.action == Intent.ACTION_VIEW) {
            openFromUri(uri)
        }
    }

    private fun openFromUri(uri: Uri) {
        val title = MarkdownSources.displayName(this, uri).ifBlank { getString(R.string.app_name) }
        val body = MarkdownSources.readText(this, uri)
        readerDirect = LoadedDocument(
            title = title,
            markdown = body ?: "# 无法打开\n\n读取该 Markdown 文件失败。",
        )
        readerDocId = null
        screen = Screen.Reader
        if (body != null) {
            lifecycleScope.launch { repository.cache(title, body, uri.toString()) }
        }
    }

    private fun openSample() {
        val markdown = assets.open("sample.md").bufferedReader().use { it.readText() }
        readerDirect = LoadedDocument(getString(R.string.app_name), markdown)
        readerDocId = null
        screen = Screen.Reader
    }

    private fun navigateToLibrary() {
        screen = Screen.Library
        readerDirect = null
        readerDocId = null
    }
}

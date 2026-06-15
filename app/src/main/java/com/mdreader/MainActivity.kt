package com.mdreader

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.lifecycleScope
import com.mdreader.data.LoadedDocument
import com.mdreader.data.MarkdownSources
import com.mdreader.ui.MDreaderTheme
import com.mdreader.ui.ReaderScreen
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    private val documentState = mutableStateOf<LoadedDocument?>(null)
    private val repository by lazy { (application as App).repository }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        handleIntent(intent)
        setContent {
            MDreaderTheme {
                val doc = documentState.value
                ReaderScreen(
                    title = doc?.title ?: getString(R.string.app_name),
                    markdown = doc?.markdown.orEmpty(),
                )
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
            return
        }
        documentState.value = bundledSample()
    }

    private fun openFromUri(uri: android.net.Uri) {
        val title = MarkdownSources.displayName(this, uri).ifBlank { getString(R.string.app_name) }
        val body = MarkdownSources.readText(this, uri)
        if (body != null) {
            documentState.value = LoadedDocument(title, body)
            lifecycleScope.launch { repository.cache(title, body, uri.toString()) }
        } else {
            documentState.value = LoadedDocument(title, "# 无法打开\n\n读取该 Markdown 文件失败。")
        }
    }

    private fun bundledSample(): LoadedDocument {
        val markdown = assets.open("sample.md").bufferedReader().use { it.readText() }
        return LoadedDocument(getString(R.string.app_name), markdown)
    }
}

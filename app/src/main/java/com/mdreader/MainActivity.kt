package com.mdreader

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.mutableStateOf
import com.mdreader.data.LoadedDocument
import com.mdreader.data.MarkdownSources
import com.mdreader.ui.MDreaderTheme
import com.mdreader.ui.ReaderScreen

class MainActivity : ComponentActivity() {

    private val documentState = mutableStateOf<LoadedDocument?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        documentState.value = documentFromIntent(intent)
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
        documentState.value = documentFromIntent(intent)
    }

    private fun documentFromIntent(intent: Intent?): LoadedDocument {
        val uri = intent?.data
        if (uri != null && intent.action == Intent.ACTION_VIEW) {
            val title = MarkdownSources.displayName(this, uri)
            val body = MarkdownSources.readText(this, uri)
                ?: "# 无法打开\n\n读取该 Markdown 文件失败。"
            return LoadedDocument(title.ifBlank { getString(R.string.app_name) }, body)
        }
        return bundledSample()
    }

    private fun bundledSample(): LoadedDocument {
        val markdown = assets.open("sample.md").bufferedReader().use { it.readText() }
        return LoadedDocument(getString(R.string.app_name), markdown)
    }
}

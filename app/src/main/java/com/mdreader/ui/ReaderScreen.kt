package com.mdreader.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.mdreader.R
import com.mdreader.data.DocRepository
import com.mdreader.render.MarkdownView

/** Full-screen reader: a top bar (with back) over the rendered [markdown]. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReaderScreen(
    title: String,
    markdown: String,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            TopAppBar(
                title = { Text(title, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(R.string.back),
                        )
                    }
                },
            )
        },
    ) { innerPadding ->
        MarkdownView(
            markdown = markdown,
            isDark = isSystemInDarkTheme(),
            modifier = Modifier.fillMaxSize().padding(innerPadding),
        )
    }
}

/** Reader entry that loads a cached document by id, showing a spinner until ready. */
@Composable
fun ReaderFromCache(
    docId: Long,
    repository: DocRepository,
    onBack: () -> Unit,
) {
    var loaded by remember(docId) { mutableStateOf<Pair<String, String>?>(null) }
    LaunchedEffect(docId) {
        loaded = repository.openDocument(docId)?.let { (entity, content) ->
            entity.title to content
        }
    }
    val current = loaded
    if (current == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
    } else {
        ReaderScreen(title = current.first, markdown = current.second, onBack = onBack)
    }
}

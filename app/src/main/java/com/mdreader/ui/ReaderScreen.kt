package com.mdreader.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.mdreader.render.MarkdownView

/** Full-screen reader: a top bar with [title] over the rendered [markdown]. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReaderScreen(
    title: String,
    markdown: String,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = { TopAppBar(title = { Text(title) }) },
    ) { innerPadding ->
        MarkdownView(
            markdown = markdown,
            isDark = isSystemInDarkTheme(),
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
        )
    }
}

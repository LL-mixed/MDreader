package com.mdreader.ui

import android.content.Context
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.List
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.mdreader.R
import com.mdreader.data.DocRepository
import com.mdreader.render.MarkdownView
import com.mdreader.render.OutlineController
import com.mdreader.render.OutlineItem
import com.mdreader.render.ZoomWebView
import com.mdreader.util.ContentHash
import kotlinx.coroutines.launch

private const val EXPANDED_WIDTH_DP = 600
private val OutlinePanelWidth = 300.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReaderScreen(
    title: String,
    markdown: String,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val isDark = isSystemInDarkTheme()
    val controller = remember { OutlineController() }
    var outline by remember { mutableStateOf<List<OutlineItem>>(emptyList()) }
    var activeIndex by remember { mutableStateOf<Int?>(null) }
    var outlineVisible by remember { mutableStateOf(true) }
    val drawerState = rememberDrawerState(initialValue = DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    val isExpanded = LocalConfiguration.current.screenWidthDp >= EXPANDED_WIDTH_DP

    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("mdreader_zoom", Context.MODE_PRIVATE) }
    var textZoom by remember(markdown) {
        mutableStateOf(prefs.getInt(ContentHash.sha256Hex(markdown), 100).coerceIn(ZoomWebView.MIN, ZoomWebView.MAX))
    }
    val onZoomChange: (Int) -> Unit = { new ->
        textZoom = new
        prefs.edit().putInt(ContentHash.sha256Hex(markdown), new).apply()
    }

    LaunchedEffect(markdown) {
        outline = emptyList()
        activeIndex = null
    }

    val onItemClick: (Int) -> Unit = { index ->
        controller.scrollToHeading(index)
        if (!isExpanded) scope.launch { drawerState.close() }
    }
    val onToggleOutline: () -> Unit = {
        if (isExpanded) {
            outlineVisible = !outlineVisible
        } else {
            scope.launch {
                if (drawerState.isOpen) drawerState.close() else drawerState.open()
            }
        }
    }

    if (isExpanded) {
        Row(modifier.fillMaxSize()) {
            if (outlineVisible) {
                Surface(
                    modifier = Modifier.width(OutlinePanelWidth).fillMaxHeight(),
                    tonalElevation = 1.dp,
                ) {
                    OutlineDrawer(
                        items = outline,
                        activeIndex = activeIndex,
                        onItemClick = onItemClick,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
            ReaderContent(
                title = title,
                markdown = markdown,
                isDark = isDark,
                controller = controller,
                textZoom = textZoom,
                onZoomChange = onZoomChange,
                onOutline = { outline = it },
                onActiveHeading = { activeIndex = it },
                onBack = onBack,
                showOutlineIcon = outline.isNotEmpty(),
                onOpenOutline = onToggleOutline,
                modifier = Modifier.weight(1f).fillMaxHeight(),
            )
        }
    } else {
        ModalNavigationDrawer(
            drawerContent = {
                OutlineDrawer(
                    items = outline,
                    activeIndex = activeIndex,
                    onItemClick = onItemClick,
                    modifier = Modifier.fillMaxSize(),
                )
            },
            drawerState = drawerState,
            gesturesEnabled = false,
            modifier = modifier,
        ) {
            ReaderContent(
                title = title,
                markdown = markdown,
                isDark = isDark,
                controller = controller,
                textZoom = textZoom,
                onZoomChange = onZoomChange,
                onOutline = { outline = it },
                onActiveHeading = { activeIndex = it },
                onBack = onBack,
                showOutlineIcon = outline.isNotEmpty(),
                onOpenOutline = onToggleOutline,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ReaderContent(
    title: String,
    markdown: String,
    isDark: Boolean,
    controller: OutlineController,
    textZoom: Int,
    onZoomChange: (Int) -> Unit,
    onOutline: (List<OutlineItem>) -> Unit,
    onActiveHeading: (Int) -> Unit,
    onBack: () -> Unit,
    showOutlineIcon: Boolean,
    onOpenOutline: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
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
                actions = {
                    if (showOutlineIcon) {
                        IconButton(onClick = onOpenOutline) {
                            Icon(
                                Icons.AutoMirrored.Filled.List,
                                contentDescription = stringResource(R.string.outline_title),
                            )
                        }
                    }
                },
            )
        },
    ) { innerPadding ->
        MarkdownView(
            markdown = markdown,
            isDark = isDark,
            controller = controller,
            textZoom = textZoom,
            onZoomChange = onZoomChange,
            onOutline = onOutline,
            onActiveHeading = onActiveHeading,
            modifier = Modifier.fillMaxSize().padding(innerPadding),
        )
    }
}

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

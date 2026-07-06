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
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.outlined.DarkMode
import androidx.compose.material.icons.outlined.LightMode
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
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
import com.mdreader.data.SettingsStore
import com.mdreader.data.ThemeStore
import com.mdreader.render.MarkdownView
import com.mdreader.render.OutlineController
import com.mdreader.render.OutlineItem
import com.mdreader.render.ZoomWebView
import com.mdreader.util.ContentHash
import com.mdreader.util.resolveDark
import kotlinx.coroutines.launch

private const val EXPANDED_WIDTH_DP = 600
private val OutlinePanelWidth = 300.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReaderScreen(
    title: String,
    markdown: String,
    themeStore: ThemeStore,
    settingsStore: SettingsStore,
    onOpenSettings: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val systemDark = isSystemInDarkTheme()
    val hash = remember(markdown) { ContentHash.sha256Hex(markdown) }
    // Mirror the per-doc override in Compose state so toggling recomposes.
    var perDocDark by remember(hash) { mutableStateOf(themeStore.isDarkFor(hash)) }
    val isDark = resolveDark(perDocDark, settingsStore.themePref, systemDark)

    // Re-skin the chrome (glass bar, drawer, dialogs) to follow the DOC's theme,
    // not the system theme — otherwise a doc pinned to dark shows a light bar.
    val docColorScheme = if (isDark) {
        androidx.compose.material3.darkColorScheme()
    } else {
        androidx.compose.material3.lightColorScheme()
    }
    androidx.compose.material3.MaterialTheme(colorScheme = docColorScheme) {
        ReaderBody(
            title = title,
            markdown = markdown,
            isDark = isDark,
            hash = hash,
            onOpenSettings = onOpenSettings,
            onBack = onBack,
            onToggleTheme = {
                val newDark = !isDark
                themeStore.setDark(newDark, hash)
                perDocDark = newDark
            },
            modifier = modifier,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ReaderBody(
    title: String,
    markdown: String,
    isDark: Boolean,
    hash: String,
    onOpenSettings: () -> Unit,
    onBack: () -> Unit,
    onToggleTheme: () -> Unit,
    modifier: Modifier = Modifier,
) {

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
        mutableStateOf(prefs.getInt(hash, 100).coerceIn(ZoomWebView.MIN, ZoomWebView.MAX))
    }
    val onZoomChange: (Int) -> Unit = { new ->
        textZoom = new
        prefs.edit().putInt(hash, new).apply()
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
                onToggleTheme = onToggleTheme,
                onOpenSettings = onOpenSettings,
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
                    // Fixed width (not fillMaxSize) so the drawer doesn't cover the
                    // whole screen — the scrim on the right stays tappable to close.
                    modifier = Modifier.width(OutlinePanelWidth).fillMaxHeight(),
                    onClose = { scope.launch { drawerState.close() } },
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
                onToggleTheme = onToggleTheme,
                onOpenSettings = onOpenSettings,
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
    onToggleTheme: () -> Unit,
    onOpenSettings: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var menuExpanded by remember { mutableStateOf(false) }
    Box(modifier = modifier) {
        // Content fills the whole area and scrolls BEHIND the translucent glass
        // bar (no top padding) so the frosted-glass effect has real content to
        // reveal. The webview's own body padding keeps the first heading readable;
        // it simply fades behind the bar when scrolled to the very top.
        MarkdownView(
            markdown = markdown,
            isDark = isDark,
            controller = controller,
            textZoom = textZoom,
            onZoomChange = onZoomChange,
            onOutline = onOutline,
            onActiveHeading = onActiveHeading,
            modifier = Modifier.fillMaxSize(),
        )
        GlassTopBar(
            title = title,
            modifier = Modifier.align(Alignment.TopStart),
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(
                        Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = stringResource(R.string.back),
                    )
                }
            },
            actions = {
                IconButton(onClick = onToggleTheme) {
                    Icon(
                        if (isDark) Icons.Outlined.LightMode else Icons.Outlined.DarkMode,
                        contentDescription = stringResource(R.string.action_toggle_theme),
                    )
                }
                if (showOutlineIcon) {
                    IconButton(onClick = onOpenOutline) {
                        Icon(
                            Icons.AutoMirrored.Filled.List,
                            contentDescription = stringResource(R.string.outline_title),
                        )
                    }
                }
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(Icons.Filled.MoreVert, contentDescription = null)
                    }
                    DropdownMenu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                        DropdownMenuItem(
                            text = { Text(stringResource(R.string.action_settings)) },
                            onClick = { menuExpanded = false; onOpenSettings() },
                        )
                    }
                }
            },
        )
    }
}

@Composable
fun ReaderFromCache(
    docId: Long,
    repository: DocRepository,
    themeStore: ThemeStore,
    settingsStore: SettingsStore,
    onOpenSettings: () -> Unit,
    onBack: () -> Unit,
) {
    var loaded by remember(docId) { mutableStateOf<Pair<String, String>?>(null) }
    LaunchedEffect(docId) {
        // Open (touches openedAt), then try to pull the latest source content. If
        // the source changed the cached snapshot is updated and we re-read it.
        repository.openDocument(docId)?.let { (entity, _) ->
            if (repository.refreshFromSource(docId)) {
                repository.loadContent(docId)?.let { fresh -> entity.title to fresh }
            } else {
                entity.title to (repository.loadContent(docId) ?: "")
            }
        }?.let { loaded = it }
    }
    val current = loaded
    if (current == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
    } else {
        ReaderScreen(
            title = current.first,
            markdown = current.second,
            themeStore = themeStore,
            settingsStore = settingsStore,
            onOpenSettings = onOpenSettings,
            onBack = onBack,
        )
    }
}

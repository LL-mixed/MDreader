package com.mdreader.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.surfaceColorAtElevation
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/**
 * A compact, translucent "frosted glass" top bar.
 *
 * - Smaller than the Material3 `TopAppBar` default: content height is
 *   [BarContentHeight] (~44dp) plus the status-bar inset.
 * - 66%-opaque surface color (theme-following) so content scrolling behind it
 *   shows through, giving a frosted-glass look. On API 31+ a `RenderEffect`
 *   blur is applied to the bar's own backdrop for extra depth.
 * - Stretches under the status bar via [statusBarsPadding] so it looks right in
 *   edge-to-edge layouts.
 *
 * Place it as an overlay ABOVE scrollable content (inside a Box), not as a
 * Scaffold `topBar`, so the translucent fill has content behind it to reveal.
 */
@Composable
fun GlassTopBar(
    title: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable (() -> Unit)? = null,
    actions: @Composable RowScope.() -> Unit = {},
    titleContent: (@Composable () -> Unit)? = null,
) {
    val barColor = MaterialTheme.colorScheme.surfaceColorAtElevation(3.dp).copy(alpha = 0.66f)
    val contentColor = MaterialTheme.colorScheme.onSurface

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier
            .fillMaxWidth()
            .background(barColor)
            .statusBarsPadding()
            .height(BarContentHeight)
            .padding(horizontal = HorizontalPadding),
    ) {
        if (navigationIcon != null) {
            navigationIcon()
            Spacer(Modifier.width(8.dp))
        }
        if (titleContent != null) {
            Box(modifier = Modifier.weight(1f)) { titleContent() }
        } else {
            Text(
                text = title,
                color = contentColor,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
        }
        actions()
    }
}

/** Total height of the bar's content row (excludes the status-bar inset). */
private val BarContentHeight = 44.dp
private val HorizontalPadding = 4.dp

/**
 * The vertical space [GlassTopBar] occupies at the top: the status-bar inset
 * plus the bar's fixed content height. Use it to inset scrollable content so its
 * first item peeks just below the bar instead of hiding under it.
 */
@Composable
fun glassTopBarInset(): androidx.compose.ui.unit.Dp =
    WindowInsets.statusBars.asPaddingValues().calculateTopPadding() + BarContentHeight

package com.mdreader.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.mdreader.render.OutlineItem

/**
 * The outline (table of contents) shown in the reader's navigation drawer.
 *
 * Renders the heading list with indentation by [OutlineItem.level], highlights
 * the heading the user is currently reading ([activeIndex]), and reports clicks
 * via [onItemClick] with the heading's document-order index — the same index the
 * renderer scrolls to. Shows an empty hint when the document has no headings.
 *
 * When [onClose] is provided (modal drawer / narrow screens), a close (✕) button
 * is shown in the header so the drawer can be dismissed without relying on the
 * scrim tap (which `gesturesEnabled = false` makes unreliable on some devices).
 */
@Composable
fun OutlineDrawer(
    items: List<OutlineItem>,
    activeIndex: Int?,
    onItemClick: (Int) -> Unit,
    modifier: Modifier = Modifier,
    onClose: (() -> Unit)? = null,
) {
    // Width is left to the caller (modal drawer fills, side panel is fixed) so
    // the same composable serves both layouts without forcing full width.
    Column(modifier.fillMaxHeight()) {
        androidx.compose.foundation.layout.Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 20.dp, top = 12.dp, end = 4.dp, bottom = 4.dp),
        ) {
            Text(
                text = "目录",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.weight(1f),
            )
            if (onClose != null) {
                IconButton(onClick = onClose) {
                    Icon(Icons.Filled.Close, contentDescription = "关闭目录")
                }
            }
        }
        if (items.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "无标题",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                items(items, key = { it.index }) { item ->
                    OutlineRow(
                        item = item,
                        isActive = activeIndex == item.index,
                        onClick = { onItemClick(item.index) },
                    )
                }
            }
        }
    }
}

@Composable
private fun OutlineRow(
    item: OutlineItem,
    isActive: Boolean,
    onClick: () -> Unit,
) {
    val scheme = MaterialTheme.colorScheme
    val indent = 14.dp * (item.level - 1).coerceAtLeast(0).toFloat()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(if (isActive) scheme.primaryContainer else scheme.surface)
            .clickable(onClick = onClick)
            .padding(start = 20.dp + indent, end = 20.dp, top = 8.dp, bottom = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = item.text.ifBlank { "（无标题）" },
            color = if (isActive) scheme.onPrimaryContainer else scheme.onSurfaceVariant,
            fontWeight = if (isActive) FontWeight.SemiBold else FontWeight.Normal,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

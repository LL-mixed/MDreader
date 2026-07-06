package com.mdreader.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.mdreader.R
import com.mdreader.data.DocRepository
import com.mdreader.data.db.CachedDocEntity
import com.mdreader.util.DateBuckets
import com.mdreader.util.DayBucket
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LibraryScreen(
    repository: DocRepository,
    onOpen: (Long) -> Unit,
    onOpenSample: () -> Unit,
) {
    val all by repository.observeAll().collectAsState(initial = emptyList())
    var query by rememberSaveable { mutableStateOf("") }
    var showSearch by rememberSaveable { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    val shown = remember(all, query) {
        if (query.isBlank()) all
        else all.filter { it.title.contains(query, ignoreCase = true) }
    }
    var pendingDelete by remember { mutableStateOf<CachedDocEntity?>(null) }

    val topInset = glassTopBarInset()
    val contentPadding = PaddingValues(top = topInset)
    Box(modifier = Modifier.fillMaxSize()) {
        when {
            all.isEmpty() -> EmptyState(Modifier.padding(top = topInset), onOpenSample)
            shown.isEmpty() -> Box(
                Modifier.fillMaxSize().padding(top = topInset),
                contentAlignment = Alignment.Center,
            ) {
                Text(stringResource(R.string.search_no_result))
            }
            else -> DocList(
                docs = shown,
                contentPadding = contentPadding,
                onOpen = onOpen,
                onToggleFavorite = { doc -> scope.launch { repository.setFavorite(doc.id, !doc.favorite) } },
                onRequestDelete = { pendingDelete = it },
            )
        }
        GlassTopBar(
            title = if (showSearch) "" else stringResource(R.string.library_title),
            modifier = Modifier.align(Alignment.TopStart),
            navigationIcon = if (showSearch) {
                {
                    IconButton(onClick = {
                        showSearch = false
                        query = ""
                    }) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.back))
                    }
                }
            } else null,
            titleContent = if (showSearch) {
                {
                    OutlinedTextField(
                        value = query,
                        onValueChange = { query = it },
                        placeholder = { Text(stringResource(R.string.search_placeholder)) },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            } else null,
            actions = {
                if (!showSearch) {
                    IconButton(onClick = {
                        showSearch = true
                    }) {
                        Icon(Icons.Filled.Search, contentDescription = stringResource(R.string.search_placeholder))
                    }
                }
            },
        )
    }

    pendingDelete?.let { doc ->
        AlertDialog(
            onDismissRequest = { pendingDelete = null },
            title = { Text(stringResource(R.string.delete_confirm_title)) },
            text = { Text(stringResource(R.string.delete_confirm_message, doc.title)) },
            confirmButton = {
                TextButton(onClick = {
                    val target = doc
                    pendingDelete = null
                    scope.launch { repository.delete(target.id) }
                }) { Text(stringResource(R.string.action_confirm)) }
            },
            dismissButton = {
                TextButton(onClick = { pendingDelete = null }) { Text(stringResource(R.string.action_cancel)) }
            },
        )
    }
}

@Composable
private fun EmptyState(modifier: Modifier, onOpenSample: () -> Unit) {
    Column(
        modifier = modifier.fillMaxSize().padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Text(stringResource(R.string.empty_title), style = MaterialTheme.typography.titleLarge)
        Spacer(Modifier.height(12.dp))
        Text(
            stringResource(R.string.empty_hint),
            textAlign = TextAlign.Center,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(24.dp))
        Button(onClick = onOpenSample) { Text(stringResource(R.string.open_sample)) }
    }
}

@Composable
private fun DocList(
    docs: List<CachedDocEntity>,
    contentPadding: androidx.compose.foundation.layout.PaddingValues,
    onOpen: (Long) -> Unit,
    onToggleFavorite: (CachedDocEntity) -> Unit,
    onRequestDelete: (CachedDocEntity) -> Unit,
) {
    val now = remember { System.currentTimeMillis() }
    val grouped = remember(docs, now) {
        docs.groupBy { DateBuckets.bucket(it.openedAt, now) }
            .toSortedMap(compareBy { orderOf(it) })
    }
    LazyColumn(modifier = Modifier.fillMaxSize(), contentPadding = contentPadding) {
        grouped.forEach { (bucket, items) ->
            item(key = "header-$bucket") { BucketHeader(bucket) }
            items(items, key = { it.id }) { doc ->
                DocRow(
                    doc = doc,
                    onOpen = { onOpen(doc.id) },
                    onToggleFavorite = { onToggleFavorite(doc) },
                    onRequestDelete = { onRequestDelete(doc) },
                )
                HorizontalDivider()
            }
        }
    }
}

private fun orderOf(bucket: DayBucket): Int = when (bucket) {
    DayBucket.TODAY -> 0
    DayBucket.YESTERDAY -> 1
    DayBucket.EARLIER -> 2
}

@Composable
private fun BucketHeader(bucket: DayBucket) {
    val label = when (bucket) {
        DayBucket.TODAY -> stringResource(R.string.bucket_today)
        DayBucket.YESTERDAY -> stringResource(R.string.bucket_yesterday)
        DayBucket.EARLIER -> stringResource(R.string.bucket_earlier)
    }
    Text(
        text = label,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.fillMaxWidth().padding(start = 16.dp, top = 12.dp, bottom = 4.dp),
    )
}

@Composable
private fun DocRow(
    doc: CachedDocEntity,
    onOpen: () -> Unit,
    onToggleFavorite: () -> Unit,
    onRequestDelete: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onOpen).padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(
                doc.title,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                DateBuckets.format(doc.openedAt),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(onClick = onToggleFavorite) {
            Icon(
                if (doc.favorite) Icons.Filled.Star else Icons.Outlined.StarBorder,
                contentDescription = stringResource(R.string.action_favorite),
            )
        }
        Box {
            IconButton(onClick = { menuExpanded = true }) {
                Icon(Icons.Filled.MoreVert, contentDescription = null)
            }
            DropdownMenu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                DropdownMenuItem(
                    text = { Text(stringResource(R.string.action_delete)) },
                    onClick = { menuExpanded = false; onRequestDelete() },
                )
            }
        }
    }
}

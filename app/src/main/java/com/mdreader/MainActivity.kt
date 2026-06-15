package com.mdreader

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.mdreader.ui.MDreaderTheme
import com.mdreader.ui.ReaderScreen

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val markdown = assets.open("sample.md").bufferedReader().use { it.readText() }
        setContent {
            MDreaderTheme {
                ReaderScreen(title = getString(R.string.app_name), markdown = markdown)
            }
        }
    }
}

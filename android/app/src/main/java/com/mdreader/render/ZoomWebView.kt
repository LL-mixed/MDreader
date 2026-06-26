package com.mdreader.render

import android.annotation.SuppressLint
import android.content.Context
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.webkit.WebView

class ZoomWebView(context: Context) : WebView(context) {
    var currentTextZoom: Int = 100
    var onZoomChange: ((Int) -> Unit)? = null

    private val scaleDetector = ScaleGestureDetector(
        context,
        object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
            override fun onScale(detector: ScaleGestureDetector): Boolean {
                currentTextZoom = (currentTextZoom * detector.scaleFactor).toInt().coerceIn(MIN, MAX)
                onZoomChange?.invoke(currentTextZoom)
                return true
            }
        },
    )

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        scaleDetector.onTouchEvent(event)
        if (scaleDetector.isInProgress) return true
        return super.onTouchEvent(event)
    }

    companion object {
        const val MIN = 30
        const val MAX = 300
    }
}

package com.example.aircompass

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform
package com.mkihr_ojisan.minecraftd_bridge.common

interface Api {
    fun <T> runOnMainThread(task: () -> T): T
    fun getPlayerCount(): Int
}
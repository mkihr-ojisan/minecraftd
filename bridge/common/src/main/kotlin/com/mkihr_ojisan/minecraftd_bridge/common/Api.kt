package com.mkihr_ojisan.minecraftd_bridge.common

interface Api {
    fun <T> runOnMainThread(task: () -> T): T
    fun logError(message: String)
    fun logInfo(message: String)
    fun getTPS(): Double?
    fun getMSPT(): Double?
    fun getPlayerCount(): Int?
    fun getEntityCount(): Int?
    fun getLoadedChunkCount(): Int?
}
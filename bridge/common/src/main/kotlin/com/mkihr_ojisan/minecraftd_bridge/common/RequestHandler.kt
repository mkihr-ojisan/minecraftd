package com.mkihr_ojisan.minecraftd_bridge.common

import com.mkihr_ojisan.minecraftd.bridge.common.protocol.*

class RequestHandler(private val api: Api) {
    fun getServerMetrics(): Protocol.ServerMetrics {
        val runtime = Runtime.getRuntime()
        val totalMemory = runtime.totalMemory()
        val freeMemory = runtime.freeMemory()

        return api.runOnMainThread {
            serverMetrics {
                api.getTPS()?.let { tps = it }
                api.getMSPT()?.let { mspt = it }
                api.getPlayerCount()?.let { playerCount = it }
                api.getEntityCount()?.let { entityCount = it }
                api.getLoadedChunkCount()?.let { loadedChunkCount = it }
                allocatedMemory = totalMemory
                usedMemory = totalMemory - freeMemory
            }
        }
    }
}
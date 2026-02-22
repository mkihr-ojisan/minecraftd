package com.mkihr_ojisan.minecraftd_bridge.common

import com.mkihr_ojisan.minecraftd.bridge.common.protocol.*

class RequestHandler(private val api: Api) {
    fun getServerMetrics(): Protocol.ServerMetrics {
        return api.runOnMainThread {
            serverMetrics {
                playerCount = api.getPlayerCount()
            }
        }
    }
}
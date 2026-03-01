package com.mkihr_ojisan.minecraftd_bridge.fabric

import com.mkihr_ojisan.minecraftd_bridge.common.Api
import net.minecraft.server.MinecraftServer
import java.util.concurrent.CompletableFuture

class FabricApi(private val server: MinecraftServer) : Api {
    private val tps = Tps()

    override fun <T> runOnMainThread(task: () -> T): T {
        val future = CompletableFuture<T>()
        server.execute {
            try {
                future.complete(task())
            } catch (e: Throwable) {
                future.completeExceptionally(e)
            }
        }
        return future.get()
    }

    override fun logError(message: String) {
        BridgeMod.logger.error(message)
    }

    override fun logInfo(message: String) {
        BridgeMod.logger.info(message)
    }

    override fun getTPS(): Double? {
        return tps.getTps()
    }

    override fun getMSPT(): Double? {
        return tps.getMspt()
    }

    override fun getPlayerCount(): Int {
        return server.playerList.playerCount
    }

    override fun getEntityCount(): Int {
        var count = 0
        for (level in server.allLevels) {
            count += level.allEntities.count()
        }
        return count
    }

    override fun getLoadedChunkCount(): Int {
        var count = 0
        for (level in server.allLevels) {
            count += level.chunkSource.loadedChunksCount
        }
        return count
    }
}
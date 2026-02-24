package com.mkihr_ojisan.minecraftd_bridge.bukkit;

import com.mkihr_ojisan.minecraftd_bridge.common.RequestHandler
import com.mkihr_ojisan.minecraftd_bridge.common.SocketServer
import org.bukkit.plugin.java.JavaPlugin;
import kotlin.concurrent.thread

@Suppress("unused")
class Plugin : JavaPlugin() {
    private val api = BukkitApi(this)
    private val socketServer = SocketServer(api)
    private val requestHandler = RequestHandler(api)

    override fun onEnable() {
        thread {
            socketServer.start(requestHandler)
        }
    }

    override fun onDisable() {
        socketServer.close()
    }
}
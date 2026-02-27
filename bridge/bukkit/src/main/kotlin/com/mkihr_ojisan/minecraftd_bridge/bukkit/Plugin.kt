package com.mkihr_ojisan.minecraftd_bridge.bukkit;

import com.mkihr_ojisan.minecraftd_bridge.common.RequestHandler
import com.mkihr_ojisan.minecraftd_bridge.common.SocketServer
import org.bukkit.plugin.java.JavaPlugin;
import kotlin.concurrent.thread

@Suppress("unused")
class Plugin : JavaPlugin() {
    private lateinit var api: BukkitApi
    private lateinit var socketServer: SocketServer
    private lateinit var requestHandler: RequestHandler

    override fun onEnable() {
        api = BukkitApi(this)
        socketServer = SocketServer(api)
        requestHandler = RequestHandler(api)

        thread {
            socketServer.start(requestHandler)
        }
    }

    override fun onDisable() {
        socketServer.close()
    }
}
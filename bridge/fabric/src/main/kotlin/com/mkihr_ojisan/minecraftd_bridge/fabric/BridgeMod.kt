package com.mkihr_ojisan.minecraftd_bridge.fabric

import com.mkihr_ojisan.minecraftd_bridge.common.RequestHandler
import com.mkihr_ojisan.minecraftd_bridge.common.SocketServer
import net.fabricmc.api.ModInitializer
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents

//? if >=1.18 {
import org.slf4j.Logger
import org.slf4j.LoggerFactory

//? } else {
/*import org.apache.logging.log4j.LogManager
import org.apache.logging.log4j.Logger

*///? }

class BridgeMod : ModInitializer {
    companion object {
        const val MOD_ID = "minecraftd_bridge"
        val logger: Logger
        //? if >=1.18 {
                = LoggerFactory.getLogger(MOD_ID)
        //? } else {
                /*= LogManager.getLogger(MOD_ID)
        *///? }
    }

    override fun onInitialize() {
        var socketServer: SocketServer? = null

        ServerLifecycleEvents.SERVER_STARTED.register {
            val api = FabricApi(it)
            val requestHandler = RequestHandler(api)
            socketServer = SocketServer(api)
            socketServer.start(requestHandler)
        }

        ServerLifecycleEvents.SERVER_STOPPED.register {
            socketServer?.close()
        }
    }
}
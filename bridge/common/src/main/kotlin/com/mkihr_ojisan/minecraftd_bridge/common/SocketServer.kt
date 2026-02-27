package com.mkihr_ojisan.minecraftd_bridge.common

import com.mkihr_ojisan.minecraftd.bridge.common.protocol.Protocol.Request
import com.mkihr_ojisan.minecraftd.bridge.common.protocol.getServerMetricsResponse
import com.mkihr_ojisan.minecraftd.bridge.common.protocol.response
import org.newsclub.net.unix.AFUNIXServerSocket
import java.io.File
import org.newsclub.net.unix.AFUNIXSocketAddress
import java.io.Closeable
import java.io.DataInputStream
import java.io.DataOutputStream
import kotlin.concurrent.thread

class SocketServer(private val api: Api): Closeable {
    private val sock = AFUNIXServerSocket.newInstance()
    val socketFile = File("minecraftd.sock")

    private var thread: Thread? = null

    fun start(requestHandler: RequestHandler) {
        if (socketFile.exists()) {
            socketFile.delete()
        }

        sock.bind(AFUNIXSocketAddress.of(socketFile))

        api.logInfo("Socket server started at ${socketFile.absolutePath}")

        this.thread = thread {
            while (true) {
                val client = sock.accept()
                thread {
                    try {
                        val input = DataInputStream(client.inputStream)
                        val output = DataOutputStream(client.outputStream)

                        while (true) {
                            val requestLength = input.readInt().toUInt()
                            val requestBytes = ByteArray(requestLength.toInt())
                            input.readFully(requestBytes)
                            val request = Request.parseFrom(requestBytes)

                            val response = when (request.payloadCase) {
                                Request.PayloadCase.PAYLOAD_NOT_SET -> {
                                    throw IllegalArgumentException("Invalid request: payload not set")
                                }

                                Request.PayloadCase.GET_SERVER_METRICS_REQUEST -> {
                                    response {
                                        getServerMetricsResponse = getServerMetricsResponse {
                                            serverMetrics = requestHandler.getServerMetrics()
                                        }
                                    }
                                }
                            }

                            val responseBytes = response.toByteArray()
                            output.writeInt(responseBytes.size)
                            output.write(responseBytes)
                            output.flush()
                        }
                    } catch (e: Exception) {
                        e.printStackTrace()
                    } finally {
                        client.close()
                    }
                }
            }
        }
    }

    override fun close() {
        thread?.interrupt()
        socketFile.delete()
    }
}

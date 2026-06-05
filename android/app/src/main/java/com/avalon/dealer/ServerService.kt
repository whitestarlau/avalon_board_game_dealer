package com.avalon.dealer

import android.app.Service
import android.content.Intent
import android.os.IBinder

class ServerService : Service() {

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val port = intent?.getIntExtra("port", 3004) ?: 3004
        val result = Server.startServer(port)
        println("Avalon server: $result")
        return START_STICKY
    }

    override fun onDestroy() {
        Server.stopServer()
        super.onDestroy()
    }
}

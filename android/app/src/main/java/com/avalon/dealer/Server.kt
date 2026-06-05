package com.avalon.dealer

object Server {
    init {
        System.loadLibrary("backend")
    }

    external fun startServer(port: Int): String
    external fun stopServer()
}

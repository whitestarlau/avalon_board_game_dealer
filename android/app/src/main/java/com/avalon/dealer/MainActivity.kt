package com.avalon.dealer

import android.content.Intent
import android.net.wifi.WifiManager
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import android.graphics.Bitmap
import android.widget.ImageView
import java.net.NetworkInterface

class MainActivity : AppCompatActivity() {

    private var isRunning = false
    private lateinit var statusText: TextView
    private lateinit var urlText: TextView
    private lateinit var toggleBtn: Button
    private lateinit var qrImage: ImageView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        statusText = findViewById(R.id.statusText)
        urlText = findViewById(R.id.urlText)
        toggleBtn = findViewById(R.id.toggleBtn)
        qrImage = findViewById(R.id.qrImage)

        toggleBtn.setOnClickListener {
            if (isRunning) {
                stopServer()
            } else {
                startServer()
            }
        }
    }

    private fun getLocalIp(): String {
        NetworkInterface.getNetworkInterfaces()?.asSequence()
            ?.flatMap { it.inetAddresses.asSequence() }
            ?.find { !it.isLoopbackAddress && it is java.net.Inet4Address }
            ?.let { return it.hostAddress }
        return "127.0.0.1"
    }

    private fun startServer() {
        val port = 3004
        val ip = getLocalIp()
        val url = "http://$ip:$port"

        Server.startServer(port)
        isRunning = true

        statusText.text = "服务器运行中"
        urlText.text = url
        toggleBtn.text = "停止服务器"

        // Generate QR code
        generateQrCode(url)
    }

    private fun stopServer() {
        Server.stopServer()
        isRunning = false
        statusText.text = "服务器已停止"
        urlText.text = ""
        toggleBtn.text = "启动服务器"
        qrImage.setImageBitmap(null)
    }

    private fun generateQrCode(url: String) {
        val writer = QRCodeWriter()
        try {
            val bitMatrix = writer.encode(url, BarcodeFormat.QR_CODE, 512, 512)
            val bitmap = Bitmap.createBitmap(512, 512, Bitmap.Config.RGB_565)
            for (x in 0 until 512) {
                for (y in 0 until 512) {
                    bitmap.setPixel(x, y,
                        if (bitMatrix[x, y]) android.graphics.Color.BLACK
                        else android.graphics.Color.WHITE
                    )
                }
            }
            qrImage.setImageBitmap(bitmap)
        } catch (e: Exception) {
            qrImage.setImageBitmap(null)
        }
    }

    override fun onDestroy() {
        if (isRunning) {
            Server.stopServer()
        }
        super.onDestroy()
    }
}

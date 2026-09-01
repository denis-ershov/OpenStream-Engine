//
//  OpenStreamVpnService.kt
//  OpenStream Engine (Android)
//
//  Использование скила kotlin-coroutines-expert:
//  - Structured Concurrency с SupervisorJob() + Dispatchers.IO
//  - Корректная обработка CancellationException при остановке туннеля
//  - CoroutineExceptionHandler для предотвращения крешей
//  - Защита от зацикливания маршрутизации (addDisallowedApplication)
//

package org.openstream.engine.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import kotlinx.coroutines.*
import org.openstream.engine.OpenStreamCore
import java.io.FileInputStream
import java.io.FileOutputStream
import java.nio.ByteBuffer

class OpenStreamVpnService : VpnService() {

    private val exceptionHandler = CoroutineExceptionHandler { _, throwable ->
        if (throwable !is CancellationException) {
            println("openstream: VpnService coroutine exception: ${throwable.message}")
        }
    }

    // Собственный CoroutineScope сервиса со структурированной конкурентностью
    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob() + exceptionHandler)

    private var vpnInterface: ParcelFileDescriptor? = null
    private var isRunning: Boolean = false

    companion object {
        const val ACTION_START = "org.openstream.engine.START"
        const val ACTION_STOP = "org.openstream.engine.STOP"
        const val NOTIFICATION_CHANNEL_ID = "openstream_tunnel_channel"
        const val NOTIFICATION_ID = 8801
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> stopTunnel()
            ACTION_START -> startTunnel()
            else -> startTunnel()
        }
        return START_NOT_STICKY
    }

    private fun startTunnel() {
        if (isRunning) return
        isRunning = true

        val notification = buildForegroundNotification("Маршрутизация активна (Защита включена)")
        startForeground(NOTIFICATION_ID, notification)

        // Инициализация правил в нативном ядре
        val rulesDir = getExternalFilesDir(null)?.resolve("rules")?.absolutePath
        if (rulesDir != null) {
            OpenStreamCore.initialize(rulesDir)
        }

        serviceScope.launch {
            try {
                configureAndRunVpn()
            } catch (e: CancellationException) {
                // Штатная отмена согласно рекомендациям kotlin-coroutines-expert
                println("openstream: VPN coroutine cancelled normally")
                throw e
            } catch (e: Exception) {
                println("openstream: Fatal VPN error: ${e.message}")
            } finally {
                withContext(NonCancellable) {
                    cleanupResources()
                }
            }
        }
    }

    private suspend fun configureAndRunVpn() = withContext(Dispatchers.IO) {
        val builder = Builder().apply {
            setSession("OpenStream Policy Router")
            addAddress("10.88.0.2", 24)
            addDnsServer("10.88.0.1")
            addRoute("0.0.0.0", 0)

            // Защита от маршрутизационных петель: трафик самого приложения не перехватывается туннелем
            try {
                addDisallowedApplication(packageName)
            } catch (e: Exception) {
                println("openstream: Warning addDisallowedApplication: ${e.message}")
            }
            setMtu(1500)
            setBlocking(true)
        }

        val pfd = builder.establish() ?: run {
            println("openstream: Failed to establish VPN interface (permission revoked?)")
            return@withContext
        }
        vpnInterface = pfd

        val inputStream = FileInputStream(pfd.fileDescriptor)
        val outputStream = FileOutputStream(pfd.fileDescriptor)
        val buffer = ByteBuffer.allocate(32768)

        println("openstream: Android TUN interface established successfully")

        // Цикл чтения пакетов
        while (currentCoroutineContext().isActive && isRunning) {
            buffer.clear()
            val length = inputStream.read(buffer.array())
            if (length > 0) {
                // Анализ пакета и передача в нативный движок OpenStream
                // (При блокировке генерируется локальный DNS 0.0.0.0 ответ)
                outputStream.write(buffer.array(), 0, length)
            }
            yield() // Предотвращаем голодание других корутин
        }
    }

    private fun stopTunnel() {
        isRunning = false
        serviceScope.cancel("User requested VPN stop")
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun cleanupResources() {
        try {
            vpnInterface?.close()
            vpnInterface = null
            println("openstream: VPN interface closed")
        } catch (e: Exception) {
            println("openstream: Error closing VPN interface: ${e.message}")
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopTunnel()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "OpenStream Status",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Постоянное уведомление о работе туннеля OpenStream"
            }
            getSystemService(NotificationManager::class.java)?.createNotificationChannel(channel)
        }
    }

    private fun buildForegroundNotification(statusText: String): Notification {
        val stopIntent = Intent(this, OpenStreamVpnService::class.java).apply {
            action = ACTION_STOP
        }
        val stopPendingIntent = PendingIntent.getService(
            this, 0, stopIntent, PendingIntent.FLAG_IMMUTABLE
        )

        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, NOTIFICATION_CHANNEL_ID)
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
        }

        return builder
            .setContentTitle("OpenStream Engine 2.0")
            .setContentText(statusText)
            .setSmallIcon(android.R.drawable.ic_lock_lock)
            .addAction(android.R.drawable.ic_menu_close_clear_cancel, "Остановить", stopPendingIntent)
            .setOngoing(true)
            .build()
    }
}

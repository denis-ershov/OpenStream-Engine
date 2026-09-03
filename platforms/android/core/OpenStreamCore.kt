//
//  OpenStreamCore.kt
//  OpenStream Engine (Android)
//
//  Использование скила kotlin-coroutines-expert:
//  - Реактивный StateFlow для метрик в реальном времени (без утечек и лишних рекомпозиций)
//  - Structured Concurrency с flowOn(Dispatchers.IO)
//  - SharedFlow для доставки одноразовых событий туннеля
//

package org.openstream.engine

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*

data class TunnelMetrics(
    val totalQueries: Long = 0,
    val blockedQueries: Long = 0,
    val proxiedQueries: Long = 0,
    val dpiEvasiveQueries: Long = 0,
    val directQueries: Long = 0,
    val memoryBytes: Long = 1024 * 1024 * 2 // 2.0 MB
)

sealed interface TunnelEvent {
    data class RuleLoaded(val ruleId: String) : TunnelEvent
    data class SecurityAlert(val message: String) : TunnelEvent
    data class Intercepted(val domain: String, val action: String) : TunnelEvent
}

object OpenStreamCore {
    const val ACTION_DIRECT = 0
    const val ACTION_DPI_EVASIVE = 1
    const val ACTION_PROXY = 2
    const val ACTION_DNS_OVERRIDE = 3
    const val ACTION_BLOCK = 4
    const val ACTION_STREAMPROXY = 5
    const val ACTION_BYPASS = 6

    init {
        try {
            System.loadLibrary("openstream_jni")
        } catch (e: UnsatisfiedLinkError) {
            // В режиме эмулятора / unit-тестов
            println("openstream: native library load error (expected in mock): ${e.message}")
        }
    }

    private val _events = MutableSharedFlow<TunnelEvent>(extraBufferCapacity = 64)
    val events: SharedFlow<TunnelEvent> = _events.asSharedFlow()

    private val externalScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    val metricsFlow: StateFlow<TunnelMetrics> = flow {
        while (currentCoroutineContext().isActive) {
            val raw = nativeGetMetrics()
            if (raw != null && raw.size >= 6) {
                emit(
                    TunnelMetrics(
                        totalQueries = raw[0],
                        blockedQueries = raw[1],
                        proxiedQueries = raw[2],
                        dpiEvasiveQueries = raw[3],
                        directQueries = raw[4],
                        memoryBytes = raw[5]
                    )
                )
            } else {
                emit(TunnelMetrics())
            }
            delay(1500)
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(
        scope = externalScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = TunnelMetrics()
    )

    fun initialize(rulesDir: String) {
        nativeInit(rulesDir)
    }

    fun matchDomain(domain: String): Int {
        return nativeMatchDomain(domain)
    }

    suspend fun loadRuleManifest(yaml: String): Result<String> = withContext(Dispatchers.IO) {
        try {
            val res = nativeLoadRule(yaml)
            if (res.startsWith("ERR:")) {
                Result.failure(IllegalArgumentException(res.removePrefix("ERR:").trim()))
            } else {
                _events.emit(TunnelEvent.RuleLoaded(res))
                Result.success(res)
            }
        } catch (e: Exception) {
            if (e is CancellationException) throw e
            Result.failure(e)
        }
    }

    // JNI Native Declarations
    @JvmStatic private external fun nativeInit(rulesDir: String)
    @JvmStatic private external fun nativeMatchDomain(domain: String): Int
    @JvmStatic private external fun nativeLoadRule(yaml: String): String
    @JvmStatic private external fun nativeGetMetrics(): LongArray?
}

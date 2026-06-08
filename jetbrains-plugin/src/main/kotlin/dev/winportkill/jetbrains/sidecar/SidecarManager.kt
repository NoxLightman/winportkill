package dev.winportkill.jetbrains.sidecar

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.CapturingProcessAdapter
import com.intellij.execution.process.OSProcessHandler
import com.intellij.execution.process.ProcessEvent
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.PathManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.thisLogger
import com.intellij.openapi.project.Project
import dev.winportkill.jetbrains.api.ApiClient
import java.net.ServerSocket
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.time.Duration
import java.time.Instant
import com.intellij.util.io.BaseOutputReader
import kotlin.io.path.exists

@Service(Service.Level.PROJECT)
class SidecarManager(
    private val project: Project,
) : Disposable {
    private val logger = thisLogger()
    private var processHandler: OSProcessHandler? = null
    private var client: ApiClient? = null

    fun ensureStarted(): ApiClient {
        client?.let { return it }

        requireWindowsX64()

        val port = pickFreePort()
        val binaryPath = resolveBinaryPath()
        // The sidecar stays silent in the common case, so use the mostly-silent reader mode
        // to avoid blocking on stdout while still capturing unexpected diagnostics.
        val handler = object : OSProcessHandler(
            GeneralCommandLine(binaryPath.toString(), "--serve", port.toString())
                .withWorkDirectory(binaryPath.parent.toFile())
        ) {
            override fun readerOptions(): BaseOutputReader.Options {
                return BaseOutputReader.Options.forMostlySilentProcess()
            }
        }

        handler.addProcessListener(object : CapturingProcessAdapter() {
            override fun onTextAvailable(event: ProcessEvent, outputType: com.intellij.openapi.util.Key<*>) {
                val text = event.text.trim()
                if (text.isNotEmpty()) {
                    logger.info("[winportkill] $text")
                }
            }

            override fun processTerminated(event: ProcessEvent) {
                logger.info("WinPortKill sidecar exited with code ${event.exitCode}")
                processHandler = null
                client = null
            }
        })

        processHandler = handler
        handler.startNotify()

        val apiClient = ApiClient("http://127.0.0.1:$port")
        waitForHealth(apiClient)
        client = apiClient
        return apiClient
    }

    override fun dispose() {
        processHandler?.destroyProcess()
        processHandler = null
        client = null
    }

    private fun resolveBinaryPath(): Path {
        // `runIde` points this property at the repo root so local development can reuse the
        // already-built sidecar instead of copying from plugin resources on every launch.
        val devRoot = System.getProperty("winportkill.dev.root")
            ?.takeIf { it.isNotBlank() }
            ?.let { path -> Path.of(path) }

        val devBinary = devRoot?.resolve(".vscode-extension/bin/win32-x64/winportkill.exe")
        if (devBinary != null && devBinary.exists()) {
            return devBinary
        }

        val pluginTempDir = Path.of(PathManager.getPluginTempPath(), "winportkill")
        Files.createDirectories(pluginTempDir)
        val target = pluginTempDir.resolve("winportkill.exe")

        val resourcePath = "/bin/win32-x64/winportkill.exe"
        javaClass.getResourceAsStream(resourcePath).use { input ->
            checkNotNull(input) { "Bundled sidecar not found: $resourcePath" }
            Files.copy(input, target, StandardCopyOption.REPLACE_EXISTING)
        }
        target.toFile().setExecutable(true)
        return target
    }

    private fun waitForHealth(apiClient: ApiClient) {
        // The process is started asynchronously by the IDE, so poll the health endpoint before
        // exposing the client to the rest of the plugin.
        val deadline = Instant.now().plus(Duration.ofSeconds(10))
        var lastError: Throwable? = null

        while (Instant.now().isBefore(deadline)) {
            try {
                if (apiClient.health().status == "ok") {
                    return
                }
            } catch (t: Throwable) {
                lastError = t
            }
            Thread.sleep(250)
        }

        processHandler?.destroyProcess()
        processHandler = null
        client = null
        error("Sidecar health check timed out${lastError?.message?.let { ": $it" } ?: ""}")
    }

    private fun pickFreePort(): Int {
        ServerSocket(0).use { socket ->
            socket.reuseAddress = true
            return socket.localPort
        }
    }

    private fun requireWindowsX64() {
        val osName = System.getProperty("os.name").lowercase()
        val osArch = System.getProperty("os.arch").lowercase()
        check(osName.contains("win")) {
            "WinPortKill JetBrains plugin currently supports only Windows x64."
        }
        check(osArch == "amd64" || osArch == "x86_64") {
            "WinPortKill JetBrains plugin currently supports only Windows x64. Detected: $osArch"
        }
    }

    companion object {
        fun getInstance(project: Project): SidecarManager = project.service()
    }
}

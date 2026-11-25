/** @file @brief Bounded foreground GATT owner; Android transport never decides decryption success. */
@file:Suppress("MissingPermission", "DEPRECATION")
package org.quantumtooth.proof

import android.Manifest
import android.bluetooth.*
import android.bluetooth.le.*
import android.content.pm.PackageManager
import android.os.*
import androidx.core.content.ContextCompat
import com.facebook.react.bridge.*
import com.facebook.react.common.LifecycleState
import com.facebook.react.module.annotations.ReactModule
import org.json.JSONObject
import java.util.UUID
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/** @brief All active state lives on worker. Callback copies are bounded and generation checked. */
@ReactModule(name = "QTooth")
class QToothModule(private val context: ReactApplicationContext) : NativeQToothSpec(context), LifecycleEventListener {
    private val thread = HandlerThread("QTooth-BLE").apply { start() }
    private val worker = Handler(thread.looper)
    private val queued = AtomicInteger(0)
    private val invalid = AtomicBoolean(false)
    private val overloaded = AtomicBoolean(false)
    @Volatile private var foreground = context.lifecycleState == LifecycleState.RESUMED
    private var epoch = 0L
    private var attempt = 0
    private var gatt: BluetoothGatt? = null
    private var scanner: BluetoothLeScanner? = null
    private var scanCallback: ScanCallback? = null
    private val devices = linkedMapOf<String, BluetoothDevice>()
    private var mtu = 23
    private var handle = 0L
    private var rx: BluetoothGattCharacteristic? = null
    private var tx: BluetoothGattCharacteristic? = null
    private var pending = ""
    private var operationUntil = Long.MAX_VALUE
    private var setupUntil = Long.MAX_VALUE
    private var scanUntil = Long.MAX_VALUE
    private var objectUntil = Long.MAX_VALUE
    private var handshakeMs = -1.0
    /** @brief Tag queued fragments so each outgoing object gets its own immutable send deadline. */
    private data class Write(val bytes: ByteArray, val objectId: Long)
    private val writes = ArrayDeque<Write>()
    private var nextObject = 0L
    private var currentObject = -1L
    private var phase = "Idle"
    private val serviceId = UUID.fromString("b6632000-7e7b-4fb5-9dc4-93f4152a0e7a")
    private val rxId = UUID.fromString("b6632001-7e7b-4fb5-9dc4-93f4152a0e7a")
    private val txId = UUID.fromString("b6632002-7e7b-4fb5-9dc4-93f4152a0e7a")
    private val cccdId = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")

    init { context.addLifecycleEventListener(this) }
    /** @brief Stable TurboModule name. */
    override fun getName() = "QTooth"
    /** @brief Android monotonic milliseconds include suspend; no wall-clock deadlines. */
    private fun now() = SystemClock.elapsedRealtime()
    /** @brief Enqueue bounded native work; overload fails closed and never resumes old callbacks. */
    private fun post(recover: Boolean = false, action: () -> Unit) {
        if (invalid.get()) return
        if (queued.incrementAndGet() > 256) {
            queued.decrementAndGet()
            if (overloaded.compareAndSet(false, true)) worker.post { stop("Native queue full · scan again", true) }
            return
        }
        worker.post {
            queued.decrementAndGet()
            if (!invalid.get() && (!overloaded.get() || recover)) {
                if (recover) overloaded.set(false)
                try { action() }
                catch (_: SecurityException) { stop("Allow Nearby devices in Settings · scan again", true) }
                catch (_: LinkageError) { stop("Native crypto unavailable · rebuild the app", true) }
                catch (_: Exception) { stop("Bluetooth/native operation failed · scan again", true) }
            }
        }
    }
    /** @brief Generation filter runs on worker; stale callbacks cannot touch a later owner. */
    private fun callback(generation: Long, action: () -> Unit) = post {
        if (generation == epoch && foreground) action()
    }
    /** @brief Emit only safe bounded fields, omitting device addresses and key-bearing data. */
    private fun event(kind: String, label: String, device: String = "", data: JSONObject? = null) {
        if (invalid.get() || (kind == "verified" && !foreground)) return
        phase = label
        val map = Arguments.createMap().apply {
            putString("kind", kind); putString("phase", label); putInt("attempt", attempt)
            putString("device", device); putString("plaintext", data?.optString("plaintext", "") ?: "")
            putString("ciphertext", data?.optString("ciphertext", "") ?: "")
            putString("sequence", data?.optString("sequence", "") ?: "")
            putInt("mtu", mtu); putInt("fragments", data?.optInt("fragments", 0) ?: 0)
            putInt("recordBytes", data?.optInt("recordBytes", 0) ?: 0)
            putDouble("handshakeMs", handshakeMs)
        }
        emitOnEvent(map)
    }
    /** @brief Check runtime permissions again natively; JS permission UI cannot bypass this gate. */
    private fun allowed(): Boolean {
        val permissions = if (Build.VERSION.SDK_INT >= 31) arrayOf(Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_CONNECT)
            else arrayOf(Manifest.permission.ACCESS_FINE_LOCATION)
        return permissions.all { ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED }
    }
    /** @brief Start a new explicit attempt and ten-second service-filtered scan, max eight peers. */
    override fun scan() {
        // A new explicit action may recover from queue overload after queued old work drains.
        if (invalid.get()) return
        post(true) {
            overloaded.set(false)
            try {
                stop("Idle", false)
                attempt++
                mtu = 23
                handshakeMs = -1.0
                event("attempt", "Checking Bluetooth")
                if (!foreground) { stop("Keep the app in the foreground · scan again", true); return@post }
                if (!allowed()) { stop("Allow Nearby devices in Settings · scan again", true); return@post }
                val adapter = context.getSystemService(BluetoothManager::class.java)?.adapter
                if (adapter == null) { stop("Bluetooth unavailable on this device", true); return@post }
                if (!adapter.isEnabled) { stop("Turn on Bluetooth in Settings · scan again", true); return@post }
                val generation = epoch
                scanner = adapter.bluetoothLeScanner
                if (scanner == null) { stop("Bluetooth scanner unavailable · scan again", true); return@post }
                scanCallback = object : ScanCallback() {
                    /** @brief Keep only bounded opaque selection tokens; never emit addresses/names. */
                    override fun onScanResult(callbackType: Int, result: ScanResult) = callback(generation) {
                        if (scanCallback == null || devices.values.any { it.address == result.device.address } || devices.size >= 8) return@callback
                        val token = "Pi ${devices.size + 1}"
                        devices[token] = result.device
                        event("device", "Scanning", token)
                    }
                    /** @brief Report scan failure without leaking backend identifiers. */
                    override fun onScanFailed(errorCode: Int) = callback(generation) { stop("Scan failed · wait briefly and scan again", true) }
                }
                scanUntil = now() + 10_000
                scanner!!.startScan(listOf(ScanFilter.Builder().setServiceUuid(ParcelUuid(serviceId)).build()),
                    ScanSettings.Builder().setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY).build(), scanCallback)
                event("status", "Scanning")
                worker.post(tick)
            } catch (_: SecurityException) { stop("Allow Nearby devices in Settings · scan again", true) }
              catch (_: Exception) { stop("Bluetooth scan failed · scan again", true) }
        }
    }
    /** @brief Connect only an opaque token from this scan; no automatic retry or reconnect. */
    override fun connect(device: String) = post {
        if (!foreground || !allowed() || gatt != null) return@post
        val peer = devices[device] ?: return@post
        stopScan()
        devices.clear()
        setupUntil = now() + 30_000
        event("status", "Connecting")
        val generation = epoch
        gatt = peer.connectGatt(context, false, gattCallbacks(generation), BluetoothDevice.TRANSPORT_LE)
        if (gatt == null) stop("Connection failed · scan again", true)
    }
    /** @brief Explicit user teardown, preserving the labelled last verified UI result. */
    override fun disconnect() = post { stop("Disconnected by user", false) }
    /** @brief Stop discovery immediately and remove its absolute deadline. */
    private fun stopScan() {
        val cb = scanCallback
        scanCallback = null
        scanUntil = Long.MAX_VALUE
        if (cb != null) try { scanner?.stopScan(cb) } catch (_: Exception) { /* @brief Teardown continues without permission. */ }
        scanner = null
    }
    /** @brief Drop all active state before best-effort transport release; no close reply is sent. */
    private fun stop(label: String, failure: Boolean) {
        epoch++
        worker.removeCallbacks(tick)
        stopScan()
        devices.clear()
        val oldHandle = handle
        handle = 0
        if (oldHandle != 0L) try { NativeCore.destroy(oldHandle) } catch (_: LinkageError) { /* @brief Missing runtime cannot revive state. */ }
        val oldGatt = gatt
        gatt = null
        rx = null; tx = null; writes.clear(); pending = ""
        nextObject = 0L; currentObject = -1L
        setupUntil = Long.MAX_VALUE; operationUntil = Long.MAX_VALUE; objectUntil = Long.MAX_VALUE
        try { oldGatt?.disconnect() } catch (_: SecurityException) { /* @brief Close still releases local GATT resources. */ }
        try { oldGatt?.close() } catch (_: Exception) { /* @brief Local state was already invalidated. */ }
        event(if (failure) "failure" else "closed", label)
        mtu = 23; handshakeMs = -1.0
    }
    /** @brief One outstanding asynchronous ATT operation, clipped by enclosing deadlines. */
    private fun begin(operation: String) {
        check(pending.isEmpty())
        pending = operation
        operationUntil = now() + 5_000
    }
    /** @brief Match completion exactly and reject failed/late operations before advancing. */
    private fun complete(operation: String, status: Int): Boolean {
        if (pending != operation || status != BluetoothGatt.GATT_SUCCESS || now() >= minOf(operationUntil, setupUntil, objectUntil)) {
            stop("Bluetooth operation failed or timed out · scan again", true)
            return false
        }
        pending = ""; operationUntil = Long.MAX_VALUE
        return true
    }
    /** @brief Generation-bound Android callback adapter; copy mutable byte arrays immediately. */
    private fun gattCallbacks(generation: Long) = object : BluetoothGattCallback() {
        /** @brief Observe link state; successful connection begins bounded MTU negotiation. */
        override fun onConnectionStateChange(link: BluetoothGatt, status: Int, newState: Int) = callback(generation) {
            if (link !== gatt) return@callback
            if (status != BluetoothGatt.GATT_SUCCESS || newState == BluetoothProfile.STATE_DISCONNECTED) {
                stop(if (status == BluetoothGatt.GATT_SUCCESS) "Disconnected · last verified result retained" else "Connection lost · scan again", status != BluetoothGatt.GATT_SUCCESS)
            } else if (newState == BluetoothProfile.STATE_CONNECTED) {
                begin("mtu")
                if (!link.requestMtu(517)) {
                    pending = ""; operationUntil = Long.MAX_VALUE
                    discover(link)
                }
            }
        }
        /** @brief Only callback MTU is an observation; the requested value is never reported as measured. */
        override fun onMtuChanged(link: BluetoothGatt, observed: Int, status: Int) = callback(generation) {
            if (link !== gatt) return@callback
            if (pending != "mtu") { stop("Unexpected MTU change · scan again", true); return@callback }
            if (now() >= operationUntil) { stop("MTU operation timed out · scan again", true); return@callback }
            pending = ""; operationUntil = Long.MAX_VALUE
            if (status == BluetoothGatt.GATT_SUCCESS) {
                if (observed !in 23..517) { stop("Unsupported MTU · scan again", true); return@callback }
                mtu = observed
            }
            discover(link)
        }
        /** @brief Require the secure RX write/TX indication properties and subscribe before crypto starts. */
        override fun onServicesDiscovered(link: BluetoothGatt, status: Int) = callback(generation) {
            if (link !== gatt || !complete("discover", status)) return@callback
            val service = link.getService(serviceId)
            rx = service?.getCharacteristic(rxId)
            tx = service?.getCharacteristic(txId)
            val input = rx
            val output = tx
            val descriptor = output?.getDescriptor(cccdId)
            if (input == null || output == null || descriptor == null ||
                input.properties and BluetoothGattCharacteristic.PROPERTY_WRITE == 0 ||
                output.properties and BluetoothGattCharacteristic.PROPERTY_INDICATE == 0 ||
                !link.setCharacteristicNotification(output, true)) {
                stop("Secure BLE service unavailable · scan again", true); return@callback
            }
            event("status", "Subscribing to indications")
            begin("subscribe")
            val ok = if (Build.VERSION.SDK_INT >= 33) link.writeDescriptor(descriptor, BluetoothGattDescriptor.ENABLE_INDICATION_VALUE) == BluetoothStatusCodes.SUCCESS
                else { descriptor.value = BluetoothGattDescriptor.ENABLE_INDICATION_VALUE; link.writeDescriptor(descriptor) }
            if (!ok) stop("Indication subscription failed · scan again", true)
        }
        /** @brief Confirm CCCD completion before creating an owner and submitting ClientHello. */
        override fun onDescriptorWrite(link: BluetoothGatt, descriptor: BluetoothGattDescriptor, status: Int) = callback(generation) {
            if (link !== gatt || descriptor.uuid != cccdId || !complete("subscribe", status)) return@callback
            handle = NativeCore.create(now(), mtu)
            if (handle == 0L) { stop("Native crypto initialization failed · rebuild the app", true); return@callback }
            native(0)
        }
        /** @brief Await every ordinary write response; only the next queued fragment may start. */
        override fun onCharacteristicWrite(link: BluetoothGatt, characteristic: BluetoothGattCharacteristic, status: Int) = callback(generation) {
            if (link !== gatt || characteristic.uuid != rxId || !complete("write", status)) return@callback
            if (writes.isEmpty()) objectUntil = Long.MAX_VALUE
            writeNext()
        }
        /** @brief Android 13+ supplies an immutable-value snapshot; copy before asynchronous ownership transfer. */
        override fun onCharacteristicChanged(link: BluetoothGatt, characteristic: BluetoothGattCharacteristic, value: ByteArray) {
            receive(link, characteristic, value, generation)
        }
        /** @brief Older Android callback needs an immediate copy of the characteristic's mutable value. */
        @Suppress("DEPRECATION")
        override fun onCharacteristicChanged(link: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
            if (Build.VERSION.SDK_INT < 33) receive(link, characteristic, characteristic.value ?: byteArrayOf(), generation)
        }
    }
    /** @brief Copy at most one ordinary ATT value before queueing, rejecting oversized input without copying it. */
    private fun receive(link: BluetoothGatt, characteristic: BluetoothGattCharacteristic, value: ByteArray, generation: Long) {
        if (value.size > 512) { callback(generation) { stop("Oversized BLE value · scan again", true) }; return }
        val bytes = value.copyOf()
        callback(generation) {
            if (link !== gatt || characteristic.uuid != txId) return@callback
            if (handle == 0L) { stop("Unexpected indication · scan again", true); return@callback }
            native(1, bytes)
        }
    }
    /** @brief Begin service discovery as one bounded setup operation. */
    private fun discover(link: BluetoothGatt) {
        event("status", "Discovering secure service")
        begin("discover")
        if (!link.discoverServices()) stop("Service discovery failed · scan again", true)
    }
    /** @brief Consume safe Rust output; verified plaintext has no other producer. */
    private fun native(operation: Int, bytes: ByteArray = byteArrayOf()) {
        if (now() >= minOf(setupUntil, operationUntil, objectUntil)) {
            stop("Bluetooth/session timed out · scan again", true); return
        }
        val value = JSONObject(NativeCore.step(handle, operation, bytes, now()))
        val kind = value.getString("kind")
        if (kind == "failure" || kind == "closed") {
            stop(value.getString("phase"), kind == "failure"); return
        }
        if (value.has("handshakeMs")) handshakeMs = value.getDouble("handshakeMs")
        if (kind == "status" || kind == "verified") {
            if (value.getString("phase").startsWith("Ready")) setupUntil = Long.MAX_VALUE
            event(kind, value.getString("phase"), data = value)
        }
        val outgoing = value.optJSONArray("writes") ?: return
        // There can be at most the remaining ClientHello fragment plus 96 PeerFinish fragments.
        check(outgoing.length() <= 128 && outgoing.length() + writes.size <= 128)
        val objectId = nextObject++
        for (i in 0 until outgoing.length()) {
            val fragment = outgoing.getJSONArray(i)
            check(fragment.length() in 9..minOf(mtu - 3, 512))
            writes.addLast(Write(ByteArray(fragment.length()) { fragment.getInt(it).toByte() }, objectId))
        }
        writeNext()
    }
    /** @brief Send one fragment with response; never use write-without-response or long writes. */
    private fun writeNext() {
        if (pending.isNotEmpty() || writes.isEmpty()) return
        val link = gatt ?: return
        val characteristic = rx ?: return
        val fragment = writes.removeFirst()
        if (currentObject != fragment.objectId) {
            currentObject = fragment.objectId
            objectUntil = now() + 10_000
        }
        val bytes = fragment.bytes
        begin("write")
        val ok = if (Build.VERSION.SDK_INT >= 33) link.writeCharacteristic(characteristic, bytes, BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT) == BluetoothStatusCodes.SUCCESS
            else {
                characteristic.value = bytes
                characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
                link.writeCharacteristic(characteristic)
            }
        if (!ok) stop("BLE write failed · scan again", true)
    }
    /** @brief Native worker watchdog enforces scan/setup/ATT/object/session bounds while JS is paused. */
    private val tick = object : Runnable {
        /** @brief Poll at 100 ms; expired state fails closed before later protocol processing. */
        override fun run() {
            if (invalid.get()) return
            try {
                val time = now()
                if (time >= scanUntil) {
                    stopScan()
                    event("status", if (devices.isEmpty()) "No Pi found · start Pi send and scan again" else "Select a Pi to connect")
                }
                if (time >= minOf(setupUntil, operationUntil, objectUntil)) {
                    stop("Bluetooth/session setup timed out · scan again", true); return
                }
                if (handle != 0L) native(2)
                if (scanCallback != null || gatt != null || devices.isNotEmpty()) worker.postDelayed(this, 100)
            } catch (_: LinkageError) { stop("Native crypto unavailable · rebuild the app", true) }
              catch (_: Exception) { stop("Native operation failed · scan again", true) }
        }
    }
    /** @brief Resume permits later explicit actions; it never reconnects. */
    override fun onHostResume() { foreground = true }
    /** @brief Stop accepting verified results immediately, then release native/GATT state in order. */
    override fun onHostPause() { foreground = false; post { stop("Foreground ended · scan again", false) } }
    /** @brief Activity destruction follows the same idempotent teardown. */
    override fun onHostDestroy() { foreground = false; post { stop("Activity closed", false) } }
    /** @brief React Native reload/invalidation rejects queued work and destroys the matching native owner. */
    override fun invalidate() {
        if (invalid.compareAndSet(false, true)) {
            foreground = false
            context.removeLifecycleEventListener(this)
            worker.post { stop("Module closed", false); thread.quitSafely() }
        }
        super.invalidate()
    }
}

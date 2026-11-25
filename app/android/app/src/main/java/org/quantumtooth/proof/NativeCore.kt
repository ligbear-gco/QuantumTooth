/** @file @brief Copied-byte JNI boundary; no pointers or secrets cross into JavaScript. */
package org.quantumtooth.proof

/** @brief Worker-only JNI access. Handles are native generations, not addresses. */
object NativeCore {
    init { System.loadLibrary("qtooth_android") }
    /** @brief Create one owner after subscription; zero is sanitized initialization failure. */
    @JvmStatic external fun create(now: Long, mtu: Int): Long
    /** @brief Advance start/receive/tick using copied bytes and Android monotonic time. */
    @JvmStatic external fun step(handle: Long, operation: Int, bytes: ByteArray, now: Long): String
    /** @brief Discard only the matching owner; stale teardown cannot affect a later session. */
    @JvmStatic external fun destroy(handle: Long)
}

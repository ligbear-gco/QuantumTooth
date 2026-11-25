/** @file @brief Register the Codegen-backed Turbo Native Module. */
package org.quantumtooth.proof

import com.facebook.react.BaseReactPackage
import com.facebook.react.bridge.NativeModule
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.model.ReactModuleInfo
import com.facebook.react.module.model.ReactModuleInfoProvider

/** @brief Lazy native module registration; no JavaScript protocol implementation. */
class QToothPackage : BaseReactPackage() {
    /** @brief Create the one known module on demand. */
    override fun getModule(name: String, reactContext: ReactApplicationContext): NativeModule? =
        if (name == "QTooth") QToothModule(reactContext) else null
    /** @brief Declare the generated TurboModule contract. */
    override fun getReactModuleInfoProvider() = ReactModuleInfoProvider {
        mapOf("QTooth" to ReactModuleInfo("QTooth", QToothModule::class.java.name, false, false, false, true))
    }
}

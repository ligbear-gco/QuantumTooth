/** @file @brief One portrait React Native activity. */
package org.quantumtooth.proof

import com.facebook.react.ReactActivity
import com.facebook.react.ReactActivityDelegate
import com.facebook.react.defaults.DefaultNewArchitectureEntryPoint.fabricEnabled
import com.facebook.react.defaults.DefaultReactActivityDelegate

/** @brief Foreground presentation; native lifecycle listener owns BLE cleanup. */
class MainActivity : ReactActivity() {
    /** @brief Resolve the packaged JavaScript component. */
    override fun getMainComponentName(): String = "QuantumTooth"
    /** @brief Use the pinned Fabric/New Architecture runtime. */
    override fun createReactActivityDelegate(): ReactActivityDelegate =
        DefaultReactActivityDelegate(this, mainComponentName, fabricEnabled)
}

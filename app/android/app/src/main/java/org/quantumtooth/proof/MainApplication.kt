/** @file @brief Embedded-only React Native host with the foreground BLE module. */
package org.quantumtooth.proof

import android.app.Application
import com.facebook.react.PackageList
import com.facebook.react.ReactApplication
import com.facebook.react.ReactHost
import com.facebook.react.ReactNativeApplicationEntryPoint.loadReactNative
import com.facebook.react.defaults.DefaultReactHost.getDefaultReactHost

/** @brief Debuggable Android process; Metro/development support is explicitly disabled. */
class MainApplication : Application(), ReactApplication {
    override val reactHost: ReactHost by lazy {
        getDefaultReactHost(
            context = applicationContext,
            packageList = PackageList(this).packages.apply { add(QToothPackage()) },
            useDevSupport = false,
        )
    }
    /** @brief Initialize the pinned React Native runtime. */
    override fun onCreate() { super.onCreate(); loadReactNative(this) }
}

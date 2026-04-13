package com.neotv.app

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.wifi.WifiManager
import android.os.Build

object WifiInfo {
    fun getNetworkId(): String {
        return try {
            val context = MainApplication.appContext ?: return "android_unknown"
            val wifiManager = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            val connectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val network = connectivityManager?.activeNetwork
                val capabilities = connectivityManager?.getNetworkCapabilities(network)
                if (capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true) {
                    val suggestionManager = context.getSystemService(Context.WIFI_SUGGESTIONS_SERVICE) as? android.net.wifi.WifiManager
                    suggestionManager?.connectedWifiInfo?.ssid?.removeSurrounding("\"") ?: "android_wifi_q"
                } else {
                    "android_not_wifi"
                }
            } else {
                @Suppress("DEPRECATION")
                val wifiInfo = wifiManager?.connectionInfo
                wifiInfo?.ssid?.removeSurrounding("\"") ?: "android_no_ssid"
            }
        } catch (e: Exception) {
            "android_error_${e.message}"
        }
    }
}

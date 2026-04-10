use serde::Deserialize;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

#[derive(Deserialize)]
struct ImmersiveModePayload {
    enabled: bool,
}

#[tauri::command]
fn set_immersive_android<R: Runtime>(
    app_handle: AppHandle<R>,
    payload: ImmersiveModePayload,
) -> Result<(), String> {
    log::info!(
        "Plugin: set_immersive_android called with enabled = {}",
        payload.enabled
    );
    #[cfg(target_os = "android")]
    {
        use tauri::AndroidExt;

        let activity = app_handle
            .android_activity()
            .ok_or("Failed to get Android activity")?;

        let vm = app_handle.android_main_thread_vm().map_err(|e| {
            log::error!("Failed to get Android VM: {:?}", e);
            e.to_string()
        })?;

        let env = vm.attach_current_thread().map_err(|e| {
            log::error!("Failed to attach JNI env: {:?}", e);
            e.to_string()
        })?;

        let plugin_class_path = "com/libretv/app/immersive/ImmersiveAndroidPlugin";
        let plugin_class = env.find_class(plugin_class_path).map_err(|e| {
            log::error!(
                "Failed to find plugin class '{}': {:?}",
                plugin_class_path,
                e
            );
            format!(
                "Failed to find plugin class '{}': {:?}",
                plugin_class_path, e
            )
        })?;

        let activity_obj: jni::objects::JObject = activity.as_jobject();

        match env.call_static_method(
            plugin_class,
            "setImmersive",
            "(Landroid/app/Activity;Z)V",
            &[activity_obj.into(), payload.enabled.into()],
        ) {
            Ok(_) => {
                log::info!("Plugin: Successfully called Kotlin setImmersive method.");
                Ok(())
            }
            Err(e) => {
                log::error!("Plugin: Error calling Kotlin setImmersive method: {:?}", e);
                Err(format!("JNI call failed: {:?}", e))
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app_handle;
        log::info!("Plugin: set_immersive_android called on non-Android OS, no action taken.");
        Ok(())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, ()> {
    Builder::<R, ()>::new("immersiveandroid")
        .invoke_handler(tauri::generate_handler![set_immersive_android])
        .build()
}
